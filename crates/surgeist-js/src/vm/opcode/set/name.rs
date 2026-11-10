use surgeist_js_ast::scope::{BindingLocator, BindingLocatorScope};

use crate::{
    Context, JsError, JsExpect, JsNativeError, JsResult,
    environments::Environment,
    vm::opcode::{IndexOperand, Operation, RegisterOperand},
};

/// `ThrowMutateImmutable` implements the Opcode Operation for `Opcode::ThrowMutateImmutable`
///
/// Operation:
///  - Throws an error because the binding access is illegal.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ThrowMutateImmutable;

impl ThrowMutateImmutable {
    #[inline(always)]
    pub(crate) fn operation(index: IndexOperand, context: &mut Context) -> JsError {
        let name = context
            .vm
            .frame()
            .code_block()
            .constant_string(index.into());

        JsNativeError::typ()
            .with_message(format!(
                "cannot mutate an immutable binding '{}'",
                name.to_std_string_escaped()
            ))
            .into()
    }
}

impl Operation for ThrowMutateImmutable {
    const NAME: &'static str = "ThrowMutateImmutable";
    const INSTRUCTION: &'static str = "INST - ThrowMutateImmutable";
    const COST: u8 = 2;
}

/// `SetName` implements the Opcode Operation for `Opcode::SetName`
///
/// Operation:
///  - Find a binding on the environment chain and assign its value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetName;

impl SetName {
    #[inline(always)]
    pub(crate) fn operation(
        (value, index): (RegisterOperand, IndexOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let value = context.vm.get_register(value.into()).clone();
        let code_block = context.vm.frame().code_block();
        let mut binding_locator = code_block.bindings[usize::from(index)].clone();
        let strict = code_block.strict();

        context.find_runtime_binding(&mut binding_locator)?;

        verify_initialized(&binding_locator, context)?;

        context.set_binding(&binding_locator, value.clone(), strict)?;

        Ok(())
    }
}

impl Operation for SetName {
    const NAME: &'static str = "SetName";
    const INSTRUCTION: &'static str = "INST - SetName";
    const COST: u8 = 4;
}

/// Sets a global block function only when this execution enabled its name.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetAnnexBGlobal;

impl SetAnnexBGlobal {
    #[inline(always)]
    pub(crate) fn operation(
        (value, name_index): (RegisterOperand, IndexOperand),
        context: &mut Context,
    ) -> JsResult<()> {
        let frame = context.vm.frame();
        let name = frame.code_block().constant_string(name_index.into());
        if !frame.annex_b_global_eligible(&name) {
            return Ok(());
        }
        let value = context.vm.get_register(value.into()).clone();
        context
            .global_object()?
            .set(name, value, false, context.access())?;
        Ok(())
    }
}

impl Operation for SetAnnexBGlobal {
    const NAME: &'static str = "SetAnnexBGlobal";
    const INSTRUCTION: &'static str = "INST - SetAnnexBGlobal";
    const COST: u8 = 4;
}

/// `SetNameByLocator` implements the Opcode Operation for `Opcode::SetNameByLocator`
///
/// Operation:
///  - Assigns a value to the binding pointed by the `current_binding` of the current frame.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SetNameByLocator;

impl SetNameByLocator {
    #[inline(always)]
    pub(crate) fn operation(value: RegisterOperand, context: &mut Context) -> JsResult<()> {
        let frame = context.vm.frame_mut();
        let strict = frame.code_block.strict();
        let mut binding_locator = frame
            .binding_stack
            .pop()
            .js_expect("locator should have been popped before")?;
        let value = context.vm.get_register(value.into()).clone();

        if context.is_deleted_binding(&binding_locator) {
            context.find_runtime_binding(&mut binding_locator)?;
        }

        verify_initialized(&binding_locator, context)?;

        context.set_binding(&binding_locator, value.clone(), strict)?;

        Ok(())
    }
}

impl Operation for SetNameByLocator {
    const NAME: &'static str = "SetNameByLocator";
    const INSTRUCTION: &'static str = "INST - SetNameByLocator";
    const COST: u8 = 4;
}

/// Checks that the binding pointed by `locator` exists and is initialized.
fn verify_initialized(locator: &BindingLocator, context: &mut Context) -> JsResult<()> {
    if !context.is_initialized_binding(locator)? {
        let key = locator.name();
        let strict = context.vm.frame().code_block.strict();

        let message = match locator.scope() {
            BindingLocatorScope::GlobalObject if strict => Some(format!(
                "cannot assign to uninitialized global property `{}`",
                key.to_std_string_escaped()
            )),
            BindingLocatorScope::GlobalObject => None,
            BindingLocatorScope::GlobalDeclarative => Some(format!(
                "cannot assign to uninitialized binding `{}`",
                key.to_std_string_escaped()
            )),
            BindingLocatorScope::Stack(index) => match context.environment_expect(index) {
                Environment::Declarative(_) => Some(format!(
                    "cannot assign to uninitialized binding `{}`",
                    key.to_std_string_escaped()
                )),
                Environment::Object(_) if strict => Some(format!(
                    "cannot assign to uninitialized property `{}`",
                    key.to_std_string_escaped()
                )),
                Environment::Object(_) => None,
            },
        };

        if let Some(message) = message {
            return Err(JsNativeError::reference().with_message(message).into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SetAnnexBGlobal;
    use crate::{
        Context, JsValue, js_string,
        property::Attribute,
        vm::{CallFrame, CodeBlock, Constant},
    };
    use std::rc::Rc;

    #[test]
    fn annex_global_assignment_uses_this_frames_eligibility_and_non_strict_set() {
        fn assign(eligible: bool, writable: bool) -> JsValue {
            let mut context = Context::builder().build().unwrap();
            let name = js_string!("block_function");
            let attributes = if writable {
                Attribute::WRITABLE
            } else {
                Attribute::empty()
            };
            context
                .register_global_property(name.clone(), 1, attributes)
                .expect("initial property");
            let mut code = CodeBlock::new(js_string!("annex"), 0, false);
            code.constants.push(Constant::String(name.clone()));
            code.register_count = 1;
            let frame = CallFrame::host(
                Rc::new(code),
                None,
                context.vm.frame().environments.clone(),
                context.realm().clone(),
            )
            .with_annex_b_eligibility(if eligible {
                vec![name.clone()]
            } else {
                Vec::new()
            });
            context
                .vm
                .push_frame_with_stack(frame, JsValue::undefined(), JsValue::null());
            context.vm.set_register(0, 41.into());
            SetAnnexBGlobal::operation((0u32.into(), 0u32.into()), &mut context)
                .expect("non-strict global assignment");
            let value = context
                .global_object()
                .unwrap()
                .get(name, context.access())
                .expect("global property");
            context.vm.pop_frame();
            value
        }

        assert_eq!(assign(false, true), 1.into());
        assert_eq!(assign(true, true), 41.into());
        assert_eq!(assign(true, false), 1.into());
    }
}
