//! Native Table part attributes and methods over canonical Source.

use super::{reflection::Receiver, table_legacy::TableFamily, *};
use surgeist_bindings_js::ValueKind;
use surgeist_dom::{
    source::{DomSourceEdit, DomSourceOutput},
    tree::DomHtmlTablePart,
};

struct Members {
    part: DomHtmlTablePart,
    attribute: &'static [u16],
    getter: &'static [u16],
    setter: &'static [u16],
    create: &'static [u16],
    delete: &'static [u16],
}

const MEMBERS: &[Members] = &[
    Members {
        part: DomHtmlTablePart::Caption,
        attribute: &ascii(b"caption"),
        getter: &ascii(b"get caption"),
        setter: &ascii(b"set caption"),
        create: &ascii(b"createCaption"),
        delete: &ascii(b"deleteCaption"),
    },
    Members {
        part: DomHtmlTablePart::Head,
        attribute: &ascii(b"tHead"),
        getter: &ascii(b"get tHead"),
        setter: &ascii(b"set tHead"),
        create: &ascii(b"createTHead"),
        delete: &ascii(b"deleteTHead"),
    },
    Members {
        part: DomHtmlTablePart::Foot,
        attribute: &ascii(b"tFoot"),
        getter: &ascii(b"get tFoot"),
        setter: &ascii(b"set tFoot"),
        create: &ascii(b"createTFoot"),
        delete: &ascii(b"deleteTFoot"),
    },
];

const TABLE: Receiver = Receiver::TableLegacy(TableFamily::Table);

#[derive(Clone, Copy)]
enum TablePartWrite {
    Set,
    Create,
    Delete,
}

pub(super) fn install_attributes<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for member in MEMBERS {
        let part = member.part;
        let read = capture(size_of::<(Capture, DomHtmlTablePart)>())?;
        let write = capture(size_of::<(Capture, DomHtmlTablePart)>())?;
        let getter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| read.read_table_part(scope, arguments, part)),
            FunctionMetadata {
                name: member.getter,
                length: 0,
            },
        )?;
        let setter = scope.fixed_arity_function(
            &Callback::new(move |scope, arguments| {
                write.write_table_part(scope, arguments, part, TablePartWrite::Set)
            }),
            FunctionMetadata {
                name: member.setter,
                length: 1,
            },
        )?;
        scope.define_new_property(
            prototype,
            PropertyKey::String(member.attribute),
            PropertyDescriptor::Accessor {
                get: Some(&getter),
                set: Some(&setter),
                enumerable: true,
                configurable: true,
            },
        )?;
    }
    Ok(())
}

pub(super) fn install_methods<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for member in MEMBERS {
        let part = member.part;
        for (name, operation) in [
            (member.create, TablePartWrite::Create),
            (member.delete, TablePartWrite::Delete),
        ] {
            let write = capture(size_of::<(Capture, DomHtmlTablePart, TablePartWrite)>())?;
            let method = scope.fixed_arity_function(
                &Callback::new(move |scope, arguments| {
                    write.write_table_part(scope, arguments, part, operation)
                }),
                FunctionMetadata { name, length: 0 },
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
    }
    Ok(())
}

impl Capture {
    fn read_table_part<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        part: DomHtmlTablePart,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(scope.type_error(TABLE.error_message())?));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let child = self.callbacks.with_read(&owner, &events, |read, _| {
                let table = read.node(id)?;
                if !TABLE.matches(table, || read.step())? {
                    return Ok(None);
                }
                Ok(Some(table.html_table_part_with(part, || read.step())?))
            })?;
            let Some(child) = child else {
                return Ok(Completion::Throw(scope.type_error(TABLE.error_message())?));
            };
            Ok(Completion::Return(self.project_observation(
                scope,
                &owner,
                &events,
                Observation::Node(child),
            )?))
        })();
        tree::tree_completion(scope, &events, result)
    }

    fn write_table_part<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        part: DomHtmlTablePart,
        operation: TablePartWrite,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(scope.type_error(TABLE.error_message())?));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let is_table = self.callbacks.with_read(&owner, &events, |read, _| {
                TABLE.matches(read.node(id)?, || read.step())
            })?;
            if !is_table {
                return Ok(Completion::Throw(scope.type_error(TABLE.error_message())?));
            }
            let edit = match operation {
                TablePartWrite::Set => {
                    let replacement = match arguments.get(0) {
                        None => None,
                        Some(value)
                            if matches!(
                                scope.kind(value)?,
                                ValueKind::Null | ValueKind::Undefined
                            ) =>
                        {
                            None
                        }
                        Some(value) => {
                            let family = match part {
                                DomHtmlTablePart::Caption => TableFamily::Caption,
                                DomHtmlTablePart::Head | DomHtmlTablePart::Foot => {
                                    TableFamily::Section
                                }
                            };
                            let receiver = Receiver::TableLegacy(family);
                            let Some(GuestNode::Element(node)) =
                                tree::projected_node(scope, value)?
                            else {
                                return Ok(Completion::Throw(
                                    scope.type_error(receiver.error_message())?,
                                ));
                            };
                            let accepted =
                                self.callbacks.with_read(&owner, &events, |read, _| {
                                    receiver.matches(read.node(node)?, || read.step())
                                })?;
                            if !accepted {
                                return Ok(Completion::Throw(
                                    scope.type_error(receiver.error_message())?,
                                ));
                            }
                            Some(node)
                        }
                    };
                    DomSourceEdit::set_html_table_part(id, part, replacement)
                }
                TablePartWrite::Delete => DomSourceEdit::delete_html_table_part(id, part),
                TablePartWrite::Create => {
                    let edit = DomSourceEdit::create_html_table_part_in_attachment(
                        id,
                        part,
                        creation_realm_fallback(&owner)?,
                    );
                    let drive = self.drive_guest_tree_edit(
                        &owner,
                        &events,
                        scope.realm_authority()?.identity(),
                        edit,
                    )?;
                    return self.finish_guest_tree_edit(
                        scope,
                        &owner,
                        &events,
                        drive.progress,
                        (drive.parent, drive.pending),
                        |scope, completion| {
                            let Some(DomSourceOutput::Node(node)) = completion.output() else {
                                return Err(DocumentScriptNativeError::ProjectionCustody.into());
                            };
                            Ok(Completion::Return(self.project_observation(
                                scope,
                                &owner,
                                &events,
                                Observation::Node(Some(node)),
                            )?))
                        },
                    );
                }
            };
            self.apply_tree_edit(scope, &owner, &events, edit, Completion::Undefined)
        })();
        tree::tree_completion(scope, &events, result)
    }
}
