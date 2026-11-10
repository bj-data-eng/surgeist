use super::RegisterOperand;
use crate::{Context, JsResult, builtins::Array, vm::opcode::Operation};

/// `RestParameterInit` implements the Opcode Operation for `Opcode::RestParameterInit`
///
/// Operation:
///  - Initialize the rest parameter value of a function from the remaining arguments.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RestParameterInit;

impl RestParameterInit {
    #[inline(always)]
    pub(super) fn operation(dst: RegisterOperand, context: &mut Context) -> JsResult<()> {
        let array = if let Some(rest) = context
            .vm
            .stack
            .pop_rest_arguments(context.vm.frames.last().expect("frame must exist"))
        {
            let rest_count = rest.len() as u32;
            context.vm.frame_mut().argument_count -= rest_count;
            // The drain is already accepted. Publish its shifted coordinates
            // before array preparation can refuse, so abort sees coherent storage.
            context.vm.frame_mut().rp -= rest_count;
            Array::create_array_from_list(rest, context)?
        } else {
            Array::array_create(0, None, context)?
        };
        context.vm.set_register(dst.into(), array.into());
        Ok(())
    }
}

impl Operation for RestParameterInit {
    const NAME: &'static str = "RestParameterInit";
    const INSTRUCTION: &'static str = "INST - RestParameterInit";
    const COST: u8 = 6;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        JsValue,
        error::{EngineError, RootingError},
        js_string,
        vm::{CallFrame, CodeBlock},
    };
    use std::rc::Rc;
    use surgeist_js_gc::heap::AccessError;

    #[test]
    fn refused_rest_array_keeps_consumed_frame_coherent_for_abort_cleanup() {
        let mut context = Context::builder().build().unwrap();
        let caller = context.vm.frame().code_block().clone();
        let mut code = CodeBlock::new(js_string!("rest-frame"), 0, false);
        code.parameter_length = 2;
        code.register_count = 1;
        let code = Rc::new(code);
        for borrowed in [true, false] {
            let checkpoint = context.vm.call_checkpoint();
            let frame = CallFrame::host(
                code.clone(),
                None,
                context.vm.frame().environments.clone(),
                context.realm().clone(),
            )
            .with_argument_count(3);
            context.vm.stack.push(JsValue::undefined());
            context.vm.stack.push(JsValue::null());
            context.vm.stack.push(11);
            context.vm.stack.push(22);
            context.vm.stack.push(33);
            context.vm.push_frame(frame);
            context.vm.set_register(0, JsValue::new(99));
            let original_rp = context.vm.frame().rp;
            let result = if borrowed {
                let runtime = context.runtime().clone();
                runtime
                    .heap
                    .read(|_| RestParameterInit::operation(0_u32.into(), &mut context))
                    .unwrap()
            } else {
                RestParameterInit::operation(0_u32.into(), &mut context)
            };
            assert_eq!(context.vm.frame().argument_count, 1);
            assert_eq!(context.vm.frame().rp, original_rp - 2);
            assert_eq!(
                context.vm.stack.get_arguments(context.vm.frame()),
                &[JsValue::new(11)]
            );
            if borrowed {
                assert_eq!(
                    result.unwrap_err().as_engine(),
                    Some(&EngineError::Rooting(RootingError::Access(
                        AccessError::HeapBorrowed
                    )))
                );
                assert_eq!(
                    context.vm.get_register(0),
                    &JsValue::new(99),
                    "refused preparation publishes no rest array"
                );
            } else {
                result.unwrap();
                let array = context.vm.get_register(0).as_object().unwrap();
                assert_eq!(array.get(0, context.access()).unwrap(), JsValue::new(22));
                assert_eq!(array.get(1, context.access()).unwrap(), JsValue::new(33));
            }
            context.vm.restore_call(checkpoint);
            assert!(Rc::ptr_eq(context.vm.frame().code_block(), &caller));
        }
    }
}
