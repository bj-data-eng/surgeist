//! Authored border styles before writing-mode or painting resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssBorderStyle, CssBoxSideKind, CssSerializationError};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl CssBorderStyle {
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_nodes(1)?;
        context.append(
            output,
            match self {
                Self::None => "none",
                Self::Hidden => "hidden",
                Self::Dotted => "dotted",
                Self::Dashed => "dashed",
                Self::Solid => "solid",
                Self::Double => "double",
                Self::Groove => "groove",
                Self::Ridge => "ridge",
                Self::Inset => "inset",
                Self::Outset => "outset",
            },
        )
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssBorderStyle {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.append_specified(&mut writer.context, &mut writer.css)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBorderStyle);

/// One or two authored flow-relative border styles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBorderStylePair {
    start: CssBorderStyle,
    authored_end: Option<CssBorderStyle>,
}

impl CssBorderStylePair {
    #[must_use]
    pub const fn new(start: CssBorderStyle, authored_end: Option<CssBorderStyle>) -> Self {
        Self {
            start,
            authored_end,
        }
    }

    pub const fn start(&self) -> &CssBorderStyle {
        &self.start
    }

    pub const fn authored_end(&self) -> Option<&CssBorderStyle> {
        self.authored_end.as_ref()
    }

    pub fn end(&self) -> &CssBorderStyle {
        self.authored_end.as_ref().unwrap_or(&self.start)
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssBorderStylePair {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;

        context.charge_nodes(1)?;
        writer.source_member(0, |writer| {
            self.start
                .append_specified(&mut writer.context, &mut writer.css)
        })?;
        if let Some(end) = &self.authored_end {
            writer.source_member(1, |writer| {
                writer.append(" ")?;
                end.append_specified(&mut writer.context, &mut writer.css)
            })?;
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBorderStylePair);

/// One to four authored physical or flow-relative border styles.
///
/// Assignments are top/right/bottom/left in physical mode and
/// block-start/inline-start/block-end/inline-end in logical mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssBorderStyleShorthand {
    kind: CssBoxSideKind,
    authored_values: Box<[CssBorderStyle]>,
}

impl CssBorderStyleShorthand {
    #[must_use]
    pub fn try_new(kind: CssBoxSideKind, values: Vec<CssBorderStyle>) -> Option<Self> {
        (1..=4).contains(&values.len()).then(|| Self {
            kind,
            authored_values: values.into_boxed_slice(),
        })
    }

    pub const fn kind(&self) -> CssBoxSideKind {
        self.kind
    }

    pub fn authored_values(&self) -> &[CssBorderStyle] {
        &self.authored_values
    }

    pub fn assigned_values(&self) -> [&CssBorderStyle; 4] {
        let v = &self.authored_values;
        match v.len() {
            1 => [&v[0], &v[0], &v[0], &v[0]],
            2 => [&v[0], &v[1], &v[0], &v[1]],
            3 => [&v[0], &v[1], &v[2], &v[1]],
            4 => [&v[0], &v[1], &v[2], &v[3]],
            _ => unreachable!("checked arity"),
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssBorderStyleShorthand {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;

        context.charge_nodes(1)?;
        if self.kind == CssBoxSideKind::Logical {
            writer.source_member(0, |writer| writer.append("logical "))?;
        }
        for (index, style) in self.authored_values.iter().enumerate() {
            writer.source_member(index, |writer| {
                if index > 0 {
                    writer.append(" ")?;
                }
                style.append_specified(&mut writer.context, &mut writer.css)
            })?;
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssBorderStyleShorthand);
