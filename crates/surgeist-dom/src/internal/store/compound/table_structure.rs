//! Native Table, TableSection and TableRow edits under one Source owner.

use super::*;
use crate::internal::{
    compound::{TableMember, TableStructureEdit},
    hosted::DomCreationRealmFallback,
};
use std::ops::ControlFlow;
use surgeist_html::vocabulary::KnownHtmlElement;
use surgeist_html_semantics::table_rows::{
    HtmlTableRowChild, HtmlTableRowOrder, HtmlTableRowReader,
};

#[derive(Clone, Copy)]
enum Members {
    Rows(HtmlTableRowOrder),
    Cells,
}

impl Members {
    fn of(member: TableMember, kind: Option<KnownHtmlElement>) -> Result<Self, DomError> {
        use KnownHtmlElement as Html;
        match (member, kind) {
            (TableMember::Row, Some(Html::Table)) => Ok(Self::Rows(HtmlTableRowOrder::Table)),
            (TableMember::Row, Some(Html::Thead | Html::Tbody | Html::Tfoot)) => {
                Ok(Self::Rows(HtmlTableRowOrder::Section))
            }
            (TableMember::Cell, Some(Html::Tr)) => Ok(Self::Cells),
            _ => Err(DomError::WrongNodeKind),
        }
    }

    fn element(self) -> KnownHtmlElement {
        match self {
            Self::Rows(_) => KnownHtmlElement::Tr,
            Self::Cells => KnownHtmlElement::Td,
        }
    }
}

enum Position {
    At(Key),
    End(Option<Key>),
}

struct Children<'a> {
    view: transaction::SequenceView<'a, NodeId, Key>,
    index: usize,
}

struct DraftRows<'a> {
    draft: &'a Draft,
    cancel: &'a AtomicBool,
}

impl<'a> HtmlTableRowReader for DraftRows<'a> {
    type Id = Key;
    type Children = Children<'a>;
    type Error = DomError;

    fn step(&mut self) -> Result<(), DomError> {
        self.draft.source_step(self.cancel)
    }

    fn children(&mut self, container: &Key) -> Result<Self::Children, DomError> {
        self.step()?;
        let view = self.draft.node(*container)?.children();
        Ok(Children { view, index: 0 })
    }

    fn next_element(
        &mut self,
        children: &mut Self::Children,
    ) -> Result<Option<HtmlTableRowChild<Key>>, DomError> {
        loop {
            self.step()?;
            let Some(id) = children.view.get(children.index) else {
                return Ok(None);
            };
            children.index += 1;
            self.step()?;
            if let transaction::KindRead::Element(element) = self.draft.node(id)?.kind() {
                let kind = KnownHtmlElement::from_name(element.name());
                return Ok(Some(HtmlTableRowChild { id, kind }));
            }
        }
    }
}

impl Draft {
    pub(super) fn edit_html_table_structure(
        &mut self,
        receiver: Key,
        edit: TableStructureEdit,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let kind = self.table_element_kind(receiver, cancel)?;
        match edit {
            TableStructureEdit::CreateBody { fallback } => {
                if kind != Some(KnownHtmlElement::Table) {
                    return Err(DomError::WrongNodeKind);
                }
                let document = self.node(receiver)?.document();
                let body = self.form_html_table_element(
                    document,
                    KnownHtmlElement::Tbody,
                    fallback,
                    limits,
                    cancel,
                )?;
                let before = self
                    .last_html_table_body(receiver, cancel)?
                    .map(|last| self.next_sibling(last, cancel))
                    .transpose()?
                    .flatten();
                self.register_ticket(body, cancel)?;
                self.insert_table_child(body, receiver, before, limits, cancel)
            }
            TableStructureEdit::Insert {
                member,
                index,
                fallback,
            } => {
                let members = Members::of(member, kind)?;
                self.insert_table_member(receiver, members, index, fallback, limits, cancel)
            }
            TableStructureEdit::Delete { member, index } => {
                let members = Members::of(member, kind)?;
                let member =
                    match self.table_member_position(receiver, members, index, false, cancel)? {
                        Position::At(key) => Some(key),
                        Position::End(last) => last,
                    };
                if let Some(member) = member {
                    self.remove(member, limits, cancel)?;
                }
                Ok(())
            }
        }
    }

    fn table_member_position(
        &self,
        receiver: Key,
        members: Members,
        index: i32,
        insert: bool,
        cancel: &AtomicBool,
    ) -> Result<Position, DomError> {
        self.source_step(cancel)?;
        let requested = match index {
            -1 => None,
            index => Some(usize::try_from(index).map_err(|_| DomError::IndexSize)?),
        };
        let mut read = DraftRows {
            draft: self,
            cancel,
        };
        let mut count = 0usize;
        let mut last = None;
        let mut member = |read: &mut DraftRows<'_>, key| {
            read.step()?;
            if requested == Some(count) {
                return Ok(ControlFlow::Break(key));
            }
            count = count
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            last = Some(key);
            Ok(ControlFlow::Continue(()))
        };
        let found = match members {
            Members::Rows(order) => order.visit(&mut read, &receiver, member)?,
            Members::Cells => {
                let mut children = read.children(&receiver)?;
                loop {
                    let Some(child) = read.next_element(&mut children)? else {
                        break ControlFlow::Continue(());
                    };
                    read.step()?;
                    if matches!(
                        child.kind,
                        Some(KnownHtmlElement::Td | KnownHtmlElement::Th)
                    ) && let ControlFlow::Break(key) = member(&mut read, child.id)?
                    {
                        break ControlFlow::Break(key);
                    }
                }
            }
        };
        match found {
            ControlFlow::Break(key) => Ok(Position::At(key)),
            ControlFlow::Continue(())
                if requested.is_none() || insert && requested == Some(count) =>
            {
                Ok(Position::End(last))
            }
            ControlFlow::Continue(()) => Err(DomError::IndexSize),
        }
    }

    fn insert_table_member(
        &mut self,
        receiver: Key,
        members: Members,
        index: i32,
        fallback: Option<DomCreationRealmFallback>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let position = self.table_member_position(receiver, members, index, true, cancel)?;
        let document = self.node(receiver)?.document();
        let child =
            self.form_html_table_element(document, members.element(), fallback, limits, cancel)?;
        self.register_ticket(child, cancel)?;
        let (parent, before) = match (position, members) {
            (Position::At(before), _) => (self.table_member_parent(before, cancel)?, Some(before)),
            (Position::End(Some(last)), Members::Rows(HtmlTableRowOrder::Table)) => {
                (self.table_member_parent(last, cancel)?, None)
            }
            (Position::End(None), Members::Rows(HtmlTableRowOrder::Table)) => {
                let body = match self.last_html_table_body(receiver, cancel)? {
                    Some(body) => body,
                    None => {
                        let body = self.form_html_table_element(
                            document,
                            KnownHtmlElement::Tbody,
                            fallback,
                            limits,
                            cancel,
                        )?;
                        // Keep the Table append beneath the detached tbody's
                        // original row insertion and any of its effects.
                        self.push_effect(
                            select::Effect::AppendImplicitTableBody {
                                table: receiver,
                                body,
                            },
                            self.effect_scoped,
                            cancel,
                        )?;
                        body
                    }
                };
                (body, None)
            }
            (Position::End(_), _) => (receiver, None),
        };
        self.insert_table_child(child, parent, before, limits, cancel)
    }

    fn last_html_table_body(
        &self,
        table: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        self.source_step(cancel)?;
        for child in self.node(table)?.children().iter().rev() {
            if self.table_element_kind(child, cancel)? == Some(KnownHtmlElement::Tbody) {
                return Ok(Some(child));
            }
        }
        Ok(None)
    }

    fn table_member_parent(&self, member: Key, cancel: &AtomicBool) -> Result<Key, DomError> {
        self.source_step(cancel)?;
        self.node(member)?.parent().ok_or(DomError::NotFound)
    }

    pub(in crate::internal::store) fn insert_table_child(
        &mut self,
        child: Key,
        parent: Key,
        before: Option<Key>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.ensure_pre_insert(child, parent, before, None, false, cancel)?;
        self.insert_validated(child, parent, before, limits, cancel)
    }
}
