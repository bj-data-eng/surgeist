// Intended as a child of the module lifecycle owner.
use super::*;
use crate::{Context, execution::ExecutionOptions};

fn context() -> Context {
    let builder = Context::builder();
    #[cfg(feature = "fuzz")]
    let builder = builder.instructions_remaining(1_000_000);
    builder.build().unwrap()
}

pub(crate) fn expected_lifecycle_debits(requests: usize) -> Vec<u64> {
    let mut debits = Vec::new();
    if requests != 0 {
        debits.push(u64::try_from(requests * std::mem::size_of::<ParentSlot>()).unwrap());
        debits.extend(std::iter::repeat_n(1, requests));
    }
    // The test-only error-owner helper names the actual payload types. This
    // expectation does not call the production layout producer.
    for bytes in crate::error::expected_cause_payloads().into_iter().chain([
        std::mem::size_of::<EvaluationData>(),
        std::mem::size_of::<EvaluationRecord>(),
    ]) {
        debits.push(1);
        debits.push(u64::try_from(bytes).unwrap());
    }
    debits
}

#[test]
fn lifecycle_slots_and_headers_have_exact_logical_byte_totals_and_atomic_refusal_prefixes() {
    for requests in [0, 1, 3] {
        let debits = expected_lifecycle_debits(requests);
        let total: u64 = debits.iter().sum();
        let declared = u64::try_from(
            requests * std::mem::size_of::<ParentSlot>()
                + requests
                + 5
                + crate::error::expected_cause_payloads()
                    .iter()
                    .sum::<usize>()
                + std::mem::size_of::<EvaluationData>()
                + std::mem::size_of::<EvaluationRecord>(),
        )
        .unwrap();
        assert_eq!(total, declared);
        let mut prefix = 0;
        for debit in debits {
            let limit = prefix + debit - 1;
            let mut context = context();
            let mut direct = None;
            let refused = context
                .with_execution(
                    ExecutionOptions::new(100_000).with_module_metadata_work_limit(limit),
                    |access| {
                        let owner = access.into_owner();
                        let result = owner.with_module_metadata(|owner| {
                            EvaluationRef::prepare(requests, &mut Control::new(owner))
                        });
                        direct = result.as_ref().err().cloned();
                        result
                    },
                )
                .unwrap();
            let direct = direct.expect("the targeted positive lifecycle debit must refuse");
            assert_eq!(
                direct.as_engine(),
                Some(&EngineError::RuntimeLimit(
                    crate::error::RuntimeLimitError::ModuleMetadataWork { limit }
                ))
            );
            assert_eq!(refused.result.err().unwrap(), direct);
            assert_eq!(
                refused.report.module_metadata_work(),
                prefix,
                "a refused byte batch does not partially spend its limit"
            );
            assert!(!refused.report.effects_started());
            prefix += debit;
        }
        let mut context = context();
        let accepted = context
            .with_execution(
                ExecutionOptions::new(100_000).with_module_metadata_work_limit(total),
                |access| {
                    access.into_owner().with_module_metadata(|owner| {
                        EvaluationRef::prepare(requests, &mut Control::new(owner))
                    })
                },
            )
            .unwrap();
        let prepared = accepted.result.unwrap();
        assert_eq!(accepted.report.module_metadata_work(), total);
        assert!(!accepted.report.effects_started());
        assert!(prepared.evaluation.cause().is_none());
        drop(prepared);
    }
}
