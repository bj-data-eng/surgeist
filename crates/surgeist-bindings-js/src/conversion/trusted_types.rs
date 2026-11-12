//! Scoped conversions for the attribute union and policy callback results.
use crate::{
    Error, ObjectPrototype, Primitive, Scope, TrustedAttributeValue, TrustedPolicyCallback,
    TrustedTypeData, TrustedTypeKind, Value,
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
        let receiver = self.scope.primitive(Primitive::Undefined);
        let receiver = self.core(receiver)?;
        let bytes = self.core(self.scope.argument_copy_storage_bytes(arguments.len()))?;
        let bytes = u64::try_from(bytes).map_err(|_| self.host(Error::Allocation))?;
        let backing = self.reserve(bytes).map_err(Failure)?;
        let result = self.scope.call(&function, &receiver, &arguments);
        drop(backing);
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
        let Converted { value, storage } = data;
        self.check_active()?;
        if !storage
            .account
            .owner
            .ptr_eq(&Rc::downgrade(&self.operations.owner_state()))
        {
            return Err(self.operations.refuse(Error::ForeignOwner).into());
        }
        let Some(value) = value else {
            return Ok(None);
        };
        let brand = value.into_brand();
        let bytes = brand
            .units
            .capacity()
            .checked_mul(size_of::<u16>())
            .ok_or(Error::Allocation)?;
        let value = self.project_native_object(prototype, brand, |header| {
            admit(header.checked_add(bytes).ok_or(Error::Allocation)?)
        })?;
        drop(storage);
        Ok(Some(value))
    }
}
