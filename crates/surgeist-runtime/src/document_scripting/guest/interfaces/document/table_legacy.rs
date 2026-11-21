//! Legacy string reflections for HTML tables, captions, sections, and rows.

use super::{
    reflection::{self, Receiver, Reflection},
    *,
};

pub(super) const ALIGN: reflection::Member = (
    &ascii(b"align"),
    &ascii(b"get align"),
    &ascii(b"set align"),
    &ascii(b"align"),
    Reflection::String,
);
pub(super) const WIDTH: reflection::Member = (
    &ascii(b"width"),
    &ascii(b"get width"),
    &ascii(b"set width"),
    &ascii(b"width"),
    Reflection::String,
);
pub(super) const CH: reflection::Member = (
    &ascii(b"ch"),
    &ascii(b"get ch"),
    &ascii(b"set ch"),
    &ascii(b"char"),
    Reflection::String,
);
pub(super) const CH_OFF: reflection::Member = (
    &ascii(b"chOff"),
    &ascii(b"get chOff"),
    &ascii(b"set chOff"),
    &ascii(b"charoff"),
    Reflection::String,
);
pub(super) const V_ALIGN: reflection::Member = (
    &ascii(b"vAlign"),
    &ascii(b"get vAlign"),
    &ascii(b"set vAlign"),
    &ascii(b"valign"),
    Reflection::String,
);

pub(super) const BG_COLOR: reflection::Member = (
    &ascii(b"bgColor"),
    &ascii(b"get bgColor"),
    &ascii(b"set bgColor"),
    &ascii(b"bgcolor"),
    Reflection::StringNullToEmpty,
);

const TABLE_REFLECTIONS: &[reflection::Member] = &[
    ALIGN,
    (
        &ascii(b"border"),
        &ascii(b"get border"),
        &ascii(b"set border"),
        &ascii(b"border"),
        Reflection::String,
    ),
    (
        &ascii(b"frame"),
        &ascii(b"get frame"),
        &ascii(b"set frame"),
        &ascii(b"frame"),
        Reflection::String,
    ),
    (
        &ascii(b"rules"),
        &ascii(b"get rules"),
        &ascii(b"set rules"),
        &ascii(b"rules"),
        Reflection::String,
    ),
    (
        &ascii(b"summary"),
        &ascii(b"get summary"),
        &ascii(b"set summary"),
        &ascii(b"summary"),
        Reflection::String,
    ),
    WIDTH,
    BG_COLOR,
    (
        &ascii(b"cellPadding"),
        &ascii(b"get cellPadding"),
        &ascii(b"set cellPadding"),
        &ascii(b"cellpadding"),
        Reflection::StringNullToEmpty,
    ),
    (
        &ascii(b"cellSpacing"),
        &ascii(b"get cellSpacing"),
        &ascii(b"set cellSpacing"),
        &ascii(b"cellspacing"),
        Reflection::StringNullToEmpty,
    ),
];

#[derive(Clone, Copy)]
pub(super) enum TableFamily {
    Table,
    Caption,
    Section,
    Row,
}

impl TableFamily {
    pub(super) fn interface(self) -> &'static str {
        match self {
            Self::Table => "HTMLTableElement",
            Self::Caption => "HTMLTableCaptionElement",
            Self::Section => "HTMLTableSectionElement",
            Self::Row => "HTMLTableRowElement",
        }
    }

    pub(super) fn error_message(self) -> &'static [u16] {
        match self {
            Self::Table => const { &ascii(b"HTMLTableElement required") },
            Self::Caption => const { &ascii(b"HTMLTableCaptionElement required") },
            Self::Section => const { &ascii(b"HTMLTableSectionElement required") },
            Self::Row => const { &ascii(b"HTMLTableRowElement required") },
        }
    }

    pub(super) fn install<'scope>(
        self,
        scope: &mut Scope<'scope>,
        prototype: &Value<'scope>,
        capture: CaptureFunding<'_>,
    ) -> Result<(), DocumentGuestEventError> {
        let members = match self {
            Self::Table => {
                super::table_part::install_attributes(scope, prototype, capture)?;
                TABLE_REFLECTIONS
            }
            Self::Caption => &[ALIGN],
            Self::Section => &[ALIGN, CH, CH_OFF, V_ALIGN],
            Self::Row => {
                super::table_row_index::install(scope, prototype, capture)?;
                &[ALIGN, CH, CH_OFF, V_ALIGN, BG_COLOR]
            }
        };
        reflection::install(
            scope,
            prototype,
            capture,
            Receiver::TableLegacy(self),
            members,
        )?;
        if matches!(self, Self::Table) {
            super::table_part::install_methods(scope, prototype, capture)?;
        }
        Ok(())
    }
}
