//! Reflections and ordinary row indexing for HTML cells and columns.

use super::{
    reflection::{self, Receiver, Reflection, TableSpan},
    *,
};
use surgeist_dom::tree::DomNode;
use surgeist_html::vocabulary::KnownHtmlElement;

const ALIGN: reflection::Member = (
    &ascii(b"align"),
    &ascii(b"get align"),
    &ascii(b"set align"),
    &ascii(b"align"),
    Reflection::String,
);
const WIDTH: reflection::Member = (
    &ascii(b"width"),
    &ascii(b"get width"),
    &ascii(b"set width"),
    &ascii(b"width"),
    Reflection::String,
);
const CH: reflection::Member = (
    &ascii(b"ch"),
    &ascii(b"get ch"),
    &ascii(b"set ch"),
    &ascii(b"char"),
    Reflection::String,
);
const CH_OFF: reflection::Member = (
    &ascii(b"chOff"),
    &ascii(b"get chOff"),
    &ascii(b"set chOff"),
    &ascii(b"charoff"),
    Reflection::String,
);
const V_ALIGN: reflection::Member = (
    &ascii(b"vAlign"),
    &ascii(b"get vAlign"),
    &ascii(b"set vAlign"),
    &ascii(b"valign"),
    Reflection::String,
);

const CELL_SPANS_AND_HEADERS: &[reflection::Member] = &[
    (
        &ascii(b"colSpan"),
        &ascii(b"get colSpan"),
        &ascii(b"set colSpan"),
        &ascii(b"colspan"),
        Reflection::TableSpan(TableSpan::CellColumns),
    ),
    (
        &ascii(b"rowSpan"),
        &ascii(b"get rowSpan"),
        &ascii(b"set rowSpan"),
        &ascii(b"rowspan"),
        Reflection::TableSpan(TableSpan::CellRows),
    ),
    (
        &ascii(b"headers"),
        &ascii(b"get headers"),
        &ascii(b"set headers"),
        &ascii(b"headers"),
        Reflection::String,
    ),
];

const CELL_REFLECTIONS: &[reflection::Member] = &[
    (
        &ascii(b"scope"),
        &ascii(b"get scope"),
        &ascii(b"set scope"),
        &ascii(b"scope"),
        Reflection::TableScope,
    ),
    (
        &ascii(b"abbr"),
        &ascii(b"get abbr"),
        &ascii(b"set abbr"),
        &ascii(b"abbr"),
        Reflection::String,
    ),
    ALIGN,
    (
        &ascii(b"axis"),
        &ascii(b"get axis"),
        &ascii(b"set axis"),
        &ascii(b"axis"),
        Reflection::String,
    ),
    (
        &ascii(b"height"),
        &ascii(b"get height"),
        &ascii(b"set height"),
        &ascii(b"height"),
        Reflection::String,
    ),
    WIDTH,
    CH,
    CH_OFF,
    (
        &ascii(b"noWrap"),
        &ascii(b"get noWrap"),
        &ascii(b"set noWrap"),
        &ascii(b"nowrap"),
        Reflection::Boolean,
    ),
    V_ALIGN,
    (
        &ascii(b"bgColor"),
        &ascii(b"get bgColor"),
        &ascii(b"set bgColor"),
        &ascii(b"bgcolor"),
        Reflection::StringNullToEmpty,
    ),
];

const COLUMN_REFLECTIONS: &[reflection::Member] = &[
    (
        &ascii(b"span"),
        &ascii(b"get span"),
        &ascii(b"set span"),
        &ascii(b"span"),
        Reflection::TableSpan(TableSpan::Column),
    ),
    ALIGN,
    CH,
    CH_OFF,
    V_ALIGN,
    WIDTH,
];

pub(super) fn install_cell<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(
        scope,
        prototype,
        capture,
        Receiver::TableCell,
        CELL_SPANS_AND_HEADERS,
    )?;
    let handler = capture(size_of::<Capture>())?;
    install_getter(
        scope,
        prototype,
        &ascii(b"cellIndex"),
        &ascii(b"get cellIndex"),
        true,
        Callback::new(move |scope, arguments| handler.table_cell_index(scope, arguments)),
    )?;
    reflection::install(
        scope,
        prototype,
        capture,
        Receiver::TableCell,
        CELL_REFLECTIONS,
    )
}

pub(super) fn install_column<'scope>(
    scope: &mut Scope<'scope>,
    prototype: &Value<'scope>,
    capture: CaptureFunding<'_>,
) -> Result<(), DocumentGuestEventError> {
    reflection::install(
        scope,
        prototype,
        capture,
        Receiver::TableColumn,
        COLUMN_REFLECTIONS,
    )
}

impl Capture {
    fn table_cell_index<'scope>(
        &self,
        scope: &mut Scope<'scope>,
        arguments: Arguments<'scope>,
    ) -> Result<Completion<'scope>, Error> {
        let Some(GuestNode::Element(id)) = tree::projected_node(scope, arguments.receiver())?
        else {
            return Ok(Completion::Throw(
                scope.type_error(Receiver::TableCell.error_message())?,
            ));
        };
        let (owner, events) = self.active(scope)?;
        let result = (|| {
            let index = self.callbacks.with_read(&owner, &events, |read, _| {
                let cell = read.node(id)?;
                if !Receiver::TableCell.matches(cell, || read.step())? {
                    return Ok(None);
                }
                Ok(Some(cell_index(read, cell)?))
            })?;
            let Some(index) = index else {
                return Ok(Completion::Throw(
                    scope.type_error(Receiver::TableCell.error_message())?,
                ));
            };
            Ok(Completion::Return(
                scope.primitive(Primitive::Number(f64::from(index)))?,
            ))
        })();
        tree::tree_completion(scope, &events, result)
    }
}

fn cell_index(read: &NativeRead<'_>, cell: DomNode<'_>) -> Result<i32, DocumentGuestEventError> {
    read.step()?;
    let Some(parent) = cell.parent() else {
        return Ok(-1);
    };
    let row = read.node(parent)?;
    if html_kind(read, row)? != Some(KnownHtmlElement::Tr) {
        return Ok(-1);
    }
    let mut index = 0_i32;
    for &child in row.children() {
        read.step()?;
        if child == cell.id() {
            return Ok(index);
        }
        if matches!(
            html_kind(read, read.node(child)?)?,
            Some(KnownHtmlElement::Td | KnownHtmlElement::Th)
        ) {
            index = index
                .checked_add(1)
                .ok_or(DocumentScriptNativeError::RepresentationExhausted)?;
        }
    }
    Err(DomError::NotFound.into())
}

pub(super) fn html_kind(
    read: &NativeRead<'_>,
    node: DomNode<'_>,
) -> Result<Option<KnownHtmlElement>, DocumentGuestEventError> {
    read.step()?;
    Ok(node
        .as_element()
        .and_then(|element| KnownHtmlElement::from_name(element.name())))
}
