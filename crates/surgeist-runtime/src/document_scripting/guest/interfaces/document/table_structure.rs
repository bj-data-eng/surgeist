//! Table body, row, and cell methods over canonical Source edits.

use super::{reflection::Receiver, table_legacy::TableFamily, *};
use surgeist_bindings_js::{
    IntegerValue,
    conversion::{Argument, ArgumentValue, IntegerType, declarations},
};
use surgeist_dom::{script::DomCreationRealmFallback, source::DomSourceEdit};

type IndexArgument = fn() -> Argument<IntegerType>;

#[derive(Clone, Copy)]
enum IndexedEdit {
    Insert(fn(NodeId, i32, DomCreationRealmFallback) -> DomSourceEdit),
    Delete(fn(NodeId, i32) -> DomSourceEdit),
}

#[derive(Clone, Copy)]
enum Edit {
    CreateBody,
    Indexed(IndexArgument, IndexedEdit),
}

const TABLE: &[(&[u16], Edit)] = &[
    (&ascii(b"createTBody"), Edit::CreateBody),
    (
        &ascii(b"insertRow"),
        Edit::Indexed(
            declarations::html_table_element_insert_row_0_index,
            IndexedEdit::Insert(DomSourceEdit::insert_html_table_row_in_attachment),
        ),
    ),
    (
        &ascii(b"deleteRow"),
        Edit::Indexed(
            declarations::html_table_element_delete_row_0_index,
            IndexedEdit::Delete(DomSourceEdit::delete_html_table_row),
        ),
    ),
];

const SECTION: &[(&[u16], Edit)] = &[
    (
        &ascii(b"insertRow"),
        Edit::Indexed(
            declarations::html_table_section_element_insert_row_0_index,
            IndexedEdit::Insert(DomSourceEdit::insert_html_table_row_in_attachment),
        ),
    ),
    (
        &ascii(b"deleteRow"),
        Edit::Indexed(
            declarations::html_table_section_element_delete_row_0_index,
            IndexedEdit::Delete(DomSourceEdit::delete_html_table_row),
        ),
    ),
];

const ROW: &[(&[u16], Edit)] = &[
    (
        &ascii(b"insertCell"),
        Edit::Indexed(
            declarations::html_table_row_element_insert_cell_0_index,
            IndexedEdit::Insert(DomSourceEdit::insert_html_table_cell_in_attachment),
        ),
    ),
    (
        &ascii(b"deleteCell"),
        Edit::Indexed(
            declarations::html_table_row_element_delete_cell_0_index,
            IndexedEdit::Delete(DomSourceEdit::delete_html_table_cell),
        ),
    ),
];

pub(super) fn install<'scope>(
    family: TableFamily,
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    let methods = match family {
        TableFamily::Table => TABLE,
        TableFamily::Caption => &[],
        TableFamily::Section => SECTION,
        TableFamily::Row => ROW,
    };
    for &(name, edit) in methods {
        let (length, maximum_arguments) = match edit {
            Edit::CreateBody => (0, 0),
            Edit::Indexed(_, IndexedEdit::Insert(_)) => (0, 1),
            Edit::Indexed(_, IndexedEdit::Delete(_)) => (1, 1),
        };
        let handler = capture(size_of::<(Capture, TableFamily, Edit)>())?;
        let method = scope.bounded_function(
            &Callback::new(move |scope, arguments| {
                handler.edit_table_structure(scope, arguments, family, edit)
            }),
            FunctionMetadata { name, length },
            maximum_arguments,
        )?;
        define_data(
            scope,
            prototype,
            PropertyKey::String(name),
            &method,
            true,
            true,
            true,
        )?;
    }
    Ok(())
}

impl Capture {
    fn edit_table_structure<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        family: TableFamily,
        edit: Edit,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::TableLegacy(family);
        let Some(GuestNode::Element(node)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let matches_receiver = || {
                self.callbacks.with_read(&owner, &events, |read, _| {
                    receiver.matches(read.node(node)?, || read.step())
                })
            };
            if !matches_receiver()? {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            let Edit::Indexed(descriptor, indexed) = edit else {
                let fallback = creation_realm_fallback(&owner)?;
                return self.apply_tree_node_edit(
                    scope,
                    &owner,
                    &events,
                    DomSourceEdit::create_html_table_body_in_attachment(node, fallback),
                );
            };
            let descriptor = descriptor();
            let input = arguments.get(descriptor.index());
            if matches!(indexed, IndexedEdit::Delete(_)) && input.is_none() {
                return Ok(Completion::Throw(
                    scope.type_error(&ascii(b"Missing table index"))?,
                ));
            }
            let converted = self
                .callbacks
                .conversion
                .run(scope, |conversion| conversion.argument(&descriptor, input))
                .map_err(Error::from)?;
            scope.check_active()?;
            let ArgumentValue::Present(IntegerValue::Long(index)) = *converted.get() else {
                return Err(Error::BackendUnavailable.into());
            };
            if !matches_receiver()? {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            }
            match indexed {
                IndexedEdit::Insert(insert) => {
                    let fallback = creation_realm_fallback(&owner)?;
                    self.apply_tree_node_edit(scope, &owner, &events, insert(node, index, fallback))
                }
                IndexedEdit::Delete(delete) => self.apply_tree_edit(
                    scope,
                    &owner,
                    &events,
                    delete(node, index),
                    Completion::Undefined,
                ),
            }
        })();
        tree::tree_completion(scope, &events, result)
    }
}
