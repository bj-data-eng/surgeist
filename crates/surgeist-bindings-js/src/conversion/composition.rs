use crate::{
    PropertyKey, Value, ValueKind,
    declarations::{ElementCreationArgument, ImportNodeArgument},
};

use super::{Conversion, Converted, Failure, IntegerType, Rule, Type, Work};

pub(super) fn element_creation_argument<'s>(
    cx: &mut Conversion<'_, 's>,
    input: &Value<'s>,
) -> Result<Converted<ElementCreationArgument>, Failure> {
    let kind = cx.scope.kind(input);
    let (value, storage) = match cx.core(kind)? {
        ValueKind::Object | ValueKind::Null | ValueKind::Undefined => {
            let Converted { value, storage } =
                cx.value(&super::declarations::element_creation_options(), input)?;
            (ElementCreationArgument::Options(value), storage)
        }
        _ => {
            let Converted { value, storage } = cx.value(&super::types::DOM_STRING, input)?;
            (ElementCreationArgument::String(value), storage)
        }
    };
    Ok(Converted { value, storage })
}

pub(super) fn import_node_argument<'s>(
    cx: &mut Conversion<'_, 's>,
    input: &Value<'s>,
) -> Result<Converted<ImportNodeArgument>, Failure> {
    let kind = cx.scope.kind(input);
    match cx.core(kind)? {
        ValueKind::Object | ValueKind::Null | ValueKind::Undefined => {
            let Converted { value, storage } =
                cx.value(&super::declarations::import_node_options(), input)?;
            Ok(Converted {
                value: ImportNodeArgument::Options(value),
                storage,
            })
        }
        _ => {
            let Converted { value, storage } = cx.value(&super::types::BOOLEAN, input)?;
            Ok(Converted {
                value: ImportNodeArgument::Subtree(value),
                storage,
            })
        }
    }
}

/// Closed typed descriptors compose without accepting downstream conversion
/// implementations or guest handles as owned conversion results.
pub trait Descriptor: sealed::Convert<Self::Output> {
    type Output;
}

pub(super) mod sealed {
    use super::{Conversion, Converted, Failure, Value};

    pub trait Convert<T> {
        fn convert<'s>(
            &self,
            cx: &mut Conversion<'_, 's>,
            value: &Value<'s>,
        ) -> Result<Converted<T>, Failure>;

        fn includes_undefined(&self) -> bool {
            false
        }
    }
}

impl<T> Descriptor for Type<T> {
    type Output = T;
}
impl<T> sealed::Convert<T> for Type<T> {
    fn convert<'s>(
        &self,
        cx: &mut Conversion<'_, 's>,
        value: &Value<'s>,
    ) -> Result<Converted<T>, Failure> {
        cx.work(Work::Value, 1).map_err(Failure)?;
        cx.typed(self, value)
    }
    fn includes_undefined(&self) -> bool {
        self.includes_undefined
    }
}
impl Descriptor for IntegerType {
    type Output = crate::IntegerValue;
}
impl sealed::Convert<crate::IntegerValue> for IntegerType {
    fn convert<'s>(
        &self,
        cx: &mut Conversion<'_, 's>,
        value: &Value<'s>,
    ) -> Result<Converted<crate::IntegerValue>, Failure> {
        cx.work(Work::Value, 1).map_err(Failure)?;
        cx.integer(*self, value)
    }
}

/// The additional null alternative, with the inner type's undefined inclusion
/// preserved. Construction is owned by nonnullable descriptor types.
///
/// ```compile_fail
/// use surgeist_bindings_js::conversion::types;
/// let invalid = types::DOM_STRING.nullable().nullable();
/// ```
pub struct Nullable<D>(pub(super) D);
impl<D: Descriptor> Nullable<D> {
    pub fn sequence(self) -> super::Sequence<Self> {
        super::Sequence(self)
    }
}
impl<D: Descriptor> Descriptor for Nullable<D> {
    type Output = Option<D::Output>;
}
impl<D: Descriptor> sealed::Convert<Option<D::Output>> for Nullable<D> {
    fn convert<'s>(
        &self,
        cx: &mut Conversion<'_, 's>,
        value: &Value<'s>,
    ) -> Result<Converted<Option<D::Output>>, Failure> {
        cx.work(Work::Value, 1).map_err(Failure)?;
        let result = cx.scope.kind(value);
        match cx.core(result)? {
            ValueKind::Null => Ok(cx.plain(None)),
            ValueKind::Undefined if !self.0.includes_undefined() => Ok(cx.plain(None)),
            _ => {
                let Converted { value, storage } = cx.value(&self.0, value)?;
                Ok(Converted {
                    value: Some(value),
                    storage,
                })
            }
        }
    }
    fn includes_undefined(&self) -> bool {
        self.0.includes_undefined()
    }
}

/// An omitted optional argument without a default remains missing. A declared
/// default and a supplied value both produce `Present`; null is a value whose
/// interpretation belongs to the selected type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgumentValue<T> {
    Missing,
    Present(T),
}

type DefaultValue<T> = for<'a, 's> fn(&mut Conversion<'a, 's>) -> Result<Converted<T>, Failure>;

/// One selected declaration's argument. Identity, index, optionality and
/// default are generated together; this is not overload selection or a native
/// interface implementation.
pub struct Argument<D: Descriptor> {
    id: &'static str,
    index: usize,
    optional: bool,
    ty: D,
    default: Option<DefaultValue<D::Output>>,
}
impl<D: Descriptor> Argument<D> {
    pub(crate) fn new(
        id: &'static str,
        index: usize,
        optional: bool,
        ty: D,
        default: Option<DefaultValue<D::Output>>,
    ) -> Self {
        Self {
            id,
            index,
            optional,
            ty,
            default,
        }
    }
    pub fn declaration(&self) -> &'static str {
        self.id
    }
    pub fn index(&self) -> usize {
        self.index
    }
}

impl<'s> Conversion<'_, 's> {
    /// Converts the selected argument value. Pass `arguments.get(ty.index())`
    /// from either callable or constructor arguments. No argument vector or
    /// native receiver is fabricated by the conversion layer.
    pub fn argument<D: Descriptor>(
        &mut self,
        ty: &Argument<D>,
        input: Option<&Value<'s>>,
    ) -> Result<Converted<ArgumentValue<D::Output>>, Failure> {
        self.work(Work::Argument, 1).map_err(Failure)?;
        let absent = match input {
            None => true,
            Some(value) if ty.optional => {
                let result = self.scope.kind(value);
                self.core(result)? == ValueKind::Undefined
            }
            Some(_) => false,
        };
        if absent {
            if let Some(default) = ty.default {
                let Converted { value, storage } = default(self)?;
                return Ok(Converted {
                    value: ArgumentValue::Present(value),
                    storage,
                });
            }
            return if ty.optional {
                Ok(self.plain(ArgumentValue::Missing))
            } else {
                Err(self.validation(Rule::RequiredArgument(ty.id)))
            };
        }
        match input {
            Some(input) => {
                let Converted { value, storage } = self.value(&ty.ty, input)?;
                Ok(Converted {
                    value: ArgumentValue::Present(value),
                    storage,
                })
            }
            None => Err(self.validation(Rule::RequiredArgument(ty.id))),
        }
    }

    pub(super) fn dictionary(
        &mut self,
        name: &'static str,
        input: &Value<'s>,
    ) -> Result<bool, Failure> {
        let result = self.scope.kind(input);
        match self.core(result)? {
            ValueKind::Undefined | ValueKind::Null => Ok(false),
            ValueKind::Object => Ok(true),
            _ => Err(self.validation(Rule::Dictionary(name))),
        }
    }

    pub(super) fn member(
        &mut self,
        input: &Value<'s>,
        object: bool,
        key: &'static [u16],
    ) -> Result<Option<Value<'s>>, Failure> {
        self.work(Work::DictionaryMember, 1).map_err(Failure)?;
        if !object {
            return Ok(None);
        }
        let result = self.scope.get(input, PropertyKey::String(key));
        let value = self.core(result)?;
        let result = self.scope.kind(&value);
        if self.core(result)? == ValueKind::Undefined {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }

    pub(super) fn null_interface_member(
        &mut self,
        name: &'static str,
        value: &Value<'s>,
    ) -> Result<bool, Failure> {
        self.work(Work::Value, 1).map_err(Failure)?;
        let result = self.scope.kind(value);
        match self.core(result)? {
            ValueKind::Null => Ok(true),
            _ => Err(self.validation(Rule::Interface(name))),
        }
    }

    pub(super) fn custom_element_registry_member(
        &mut self,
        value: &Value<'s>,
    ) -> Result<Option<crate::CustomElementRegistryIdentity>, Failure> {
        self.work(Work::Value, 1).map_err(Failure)?;
        let kind = self.scope.kind(value);
        if self.core(kind)? == ValueKind::Null {
            return Ok(None);
        }
        let token = self.scope.custom_element_registry_identity(value);
        self.core(token)?
            .map(Some)
            .ok_or_else(|| self.validation(Rule::Interface("CustomElementRegistry")))
    }

    pub(super) fn required_custom_element_registry_member(
        &mut self,
        value: &Value<'s>,
    ) -> Result<crate::CustomElementRegistryIdentity, Failure> {
        self.custom_element_registry_member(value)?
            .ok_or_else(|| self.validation(Rule::Interface("CustomElementRegistry")))
    }
}
