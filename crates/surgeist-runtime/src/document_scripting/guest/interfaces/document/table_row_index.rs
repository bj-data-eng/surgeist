//! Scalar Row indexes over the current ordinary native table tree.

use super::{reflection::Receiver, table_legacy::TableFamily, *};
use surgeist_html_semantics::table_rows::HtmlTableRowIndexKind;

pub(super) fn install<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    for (name, getter, kind) in [
        (
            &ascii(b"rowIndex")[..],
            &ascii(b"get rowIndex")[..],
            HtmlTableRowIndexKind::Table,
        ),
        (
            &ascii(b"sectionRowIndex")[..],
            &ascii(b"get sectionRowIndex")[..],
            HtmlTableRowIndexKind::Section,
        ),
    ] {
        let read = capture(size_of::<(Capture, HtmlTableRowIndexKind)>())?;
        install_getter(
            scope,
            prototype,
            name,
            getter,
            true,
            Callback::new(move |scope, arguments| read.read_row_index(scope, arguments, kind)),
        )?;
    }
    Ok(())
}

impl Capture {
    fn read_row_index<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
        kind: HtmlTableRowIndexKind,
    ) -> Result<Completion<'scope>, Error> {
        let receiver = Receiver::TableLegacy(TableFamily::Row);
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(receiver.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            events.step()?;
            let index = self.callbacks.with_read(&owner, &events, |read, _| {
                let row = read.node(id)?;
                if !receiver.matches(row, || read.step())? {
                    return Ok(None);
                }
                let index = row.html_table_row_index_with(kind, || read.step())?;
                let index = index
                    .map(i32::try_from)
                    .transpose()
                    .map_err(|_| DomError::RepresentationExhausted)?;
                Ok(Some(index.unwrap_or(-1)))
            })?;
            let Some(index) = index else {
                return Ok(Completion::Throw(
                    scope.type_error(receiver.error_message())?,
                ));
            };
            Ok(Completion::Return(
                scope.primitive(Primitive::Number(f64::from(index)))?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }
}
