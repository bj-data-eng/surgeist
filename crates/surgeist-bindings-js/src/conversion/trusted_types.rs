//! Scoped conversions for the attribute union and policy callback results.
use crate::{
    DomString, Error, ObjectPrototype, Primitive, Scope, TrustedAttributeValue,
    TrustedPolicyCallback, TrustedTypeData, TrustedTypeKind, UsvString, Value,
};
use std::rc::Rc;

use super::{Conversion, Converted, Failure, Nullable, Type, Work, dom_string, usv_string};

impl<'scope> Conversion<'_, 'scope> {
    /// Invoke a selected default-policy option with the original input, the
    /// expected interface name, and the sink name. The caller retains any
    /// native callback scope through this call and its return coercion.
    pub fn invoke_default_policy(
        &mut self,
        callback: &TrustedPolicyCallback,
        kind: TrustedTypeKind,
        input: &[u16],
        sink: &[u16],
    ) -> Result<Converted<Option<TrustedTypeData>>, Failure> {
        let function = self.scope.open_callback_object(&callback.0);
        let function = self.core(function)?;
        self.work(Work::Argument, 3).map_err(Failure)?;
        let type_name = match kind {
            TrustedTypeKind::Html => &b"TrustedHTML".map(u16::from)[..],
            TrustedTypeKind::Script => &b"TrustedScript".map(u16::from)[..],
            TrustedTypeKind::ScriptUrl => &b"TrustedScriptURL".map(u16::from)[..],
        };
        let arguments = [
            self.copy_to_js(input)?,
            self.copy_to_js(type_name)?,
            self.copy_to_js(sink)?,
        ];
        self.invoke_policy(&function, kind, &arguments, &[])
    }

    /// Invoke a named policy method after its required DOMString conversion.
    /// The trailing arguments retain their original guest values and order.
    /// A null or undefined callback result supplies empty branded data.
    pub fn invoke_named_policy(
        &mut self,
        callback: &TrustedPolicyCallback,
        kind: TrustedTypeKind,
        input: &DomString,
        trailing: &[Value<'scope>],
    ) -> Result<Converted<TrustedTypeData>, Failure> {
        let function = self.scope.open_callback_object(&callback.0);
        let function = self.core(function)?;
        let count = u64::try_from(trailing.len())
            .ok()
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| self.host(Error::TooManyArguments))?;
        self.work(Work::Argument, count).map_err(Failure)?;
        let input = self.copy_to_js(input.as_code_units())?;
        let Converted { value, storage } =
            self.invoke_policy(&function, kind, &[input], trailing)?;
        let value = value.unwrap_or_else(|| match kind {
            TrustedTypeKind::Html => TrustedTypeData::Html(DomString::new(Vec::new())),
            TrustedTypeKind::Script => TrustedTypeData::Script(DomString::new(Vec::new())),
            TrustedTypeKind::ScriptUrl => {
                TrustedTypeData::ScriptUrl(UsvString::normalize(Vec::new()))
            }
        });
        Ok(Converted { value, storage })
    }

    fn invoke_policy(
        &mut self,
        function: &Value<'scope>,
        kind: TrustedTypeKind,
        prefix: &[Value<'scope>],
        trailing: &[Value<'scope>],
    ) -> Result<Converted<Option<TrustedTypeData>>, Failure> {
        let receiver = self.scope.primitive(Primitive::Undefined);
        let receiver = self.core(receiver)?;
        let result = self
            .scope
            .call_with_prefix(function, &receiver, prefix, trailing);
        let result = self.core(result)?;
        self.value(&trusted_policy_callback_result(kind), &result)
    }
}

pub(super) fn attribute_value<'scope>(
    cx: &mut Conversion<'_, 'scope>,
    input: &Value<'scope>,
) -> Result<Converted<TrustedAttributeValue>, Failure> {
    let branded = cx.scope.trusted_type(input);
    if let Some(branded) = cx.core(branded)? {
        return Ok(cx.plain(TrustedAttributeValue::Trusted(branded)));
    }
    let Converted { value, storage } = dom_string(cx, input)?;
    Ok(Converted {
        value: TrustedAttributeValue::String(value),
        storage,
    })
}

/// The selected nullable return conversion for a Trusted Type policy callback.
/// JavaScript coercion and any abrupt outcome remain in the entered scope.
pub fn trusted_policy_callback_result(kind: TrustedTypeKind) -> Nullable<Type<TrustedTypeData>> {
    Type::scalar(match kind {
        TrustedTypeKind::Html => html,
        TrustedTypeKind::Script => script,
        TrustedTypeKind::ScriptUrl => script_url,
    })
    .nullable()
}

fn html<'scope>(
    cx: &mut Conversion<'_, 'scope>,
    input: &Value<'scope>,
) -> Result<Converted<TrustedTypeData>, Failure> {
    let Converted { value, storage } = dom_string(cx, input)?;
    Ok(Converted {
        value: TrustedTypeData::Html(value),
        storage,
    })
}

fn script<'scope>(
    cx: &mut Conversion<'_, 'scope>,
    input: &Value<'scope>,
) -> Result<Converted<TrustedTypeData>, Failure> {
    let Converted { value, storage } = dom_string(cx, input)?;
    Ok(Converted {
        value: TrustedTypeData::Script(value),
        storage,
    })
}

fn script_url<'scope>(
    cx: &mut Conversion<'_, 'scope>,
    input: &Value<'scope>,
) -> Result<Converted<TrustedTypeData>, Failure> {
    let Converted { value, storage } = usv_string(cx, input)?;
    Ok(Converted {
        value: TrustedTypeData::ScriptUrl(value),
        storage,
    })
}

impl<'scope> Scope<'scope> {
    /// Project the exact converted policy value using its native brand.
    /// Admission includes its UTF-16 backing before the conversion charge ends.
    /// A nullable result supplies no object. The policy caller decides what
    /// that result means for the particular invocation.
    pub fn project_trusted_type<Storage: 'static, E: From<Error>>(
        &mut self,
        prototype: ObjectPrototype<'_, 'scope>,
        data: Converted<Option<TrustedTypeData>>,
        admit: impl FnOnce(usize) -> Result<Storage, E>,
    ) -> Result<Option<Value<'scope>>, E> {
        let Converted { value, storage } = self.qualify_trusted_data(data)?;
        let result = value
            .map(|value| self.project_trusted_data(prototype, value, admit))
            .transpose();
        drop(storage);
        result
    }

    /// Project the present result of a named policy call.
    pub fn project_named_trusted_type<Storage: 'static, E: From<Error>>(
        &mut self,
        prototype: ObjectPrototype<'_, 'scope>,
        data: Converted<TrustedTypeData>,
        admit: impl FnOnce(usize) -> Result<Storage, E>,
    ) -> Result<Value<'scope>, E> {
        let Converted { value, storage } = self.qualify_trusted_data(data)?;
        let result = self.project_trusted_data(prototype, value, admit);
        drop(storage);
        result
    }

    fn qualify_trusted_data<T>(&mut self, data: Converted<T>) -> Result<Converted<T>, Error> {
        self.check_active()?;
        if !data
            .storage
            .account
            .owner
            .ptr_eq(&Rc::downgrade(&self.operations.owner_state()))
        {
            return Err(self.operations.refuse(Error::ForeignOwner));
        }
        Ok(data)
    }

    fn project_trusted_data<Storage: 'static, E: From<Error>>(
        &mut self,
        prototype: ObjectPrototype<'_, 'scope>,
        value: TrustedTypeData,
        admit: impl FnOnce(usize) -> Result<Storage, E>,
    ) -> Result<Value<'scope>, E> {
        let brand = value.into_brand();
        let bytes = brand
            .units
            .capacity()
            .checked_mul(size_of::<u16>())
            .ok_or(Error::Allocation)?;
        self.project_native_object(prototype, brand, |header| {
            admit(header.checked_add(bytes).ok_or(Error::Allocation)?)
        })
    }
}
