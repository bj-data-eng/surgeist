//! The pool work the adapter admits, compared with the HTML accounting schema's
//! credits for the same charge.

use super::*;
use surgeist_html::{
    accounting::{HtmlLimits, charging_tariffs},
    bounded::values::HtmlStringPool,
};
use surgeist_ledger::{Error, Operation, ScopeCharge};

/// The steps the work owner takes when the adapter admits `charge`.
fn steps(charge: impl FnOnce(&mut PoolCharge<'_, '_>) -> Result<(), DomError>) -> u64 {
    let mut steps = 0_u64;
    charge(&mut PoolCharge::new(&mut Admission {
        global: &mut |_| Ok(()),
        step: &mut || {
            steps += 1;
            Ok(())
        },
    }))
    .unwrap();
    steps
}

/// The credits the schema records at the cap scope when `charge` charges a
/// ledger scope below it.
fn credits(
    charge: impl FnOnce(
        &mut ScopeCharge<'_, HtmlWork, HtmlResource>,
    ) -> Result<(), Error<HtmlWork, HtmlResource>>,
) -> u64 {
    let mut operation = Operation::new("count", HtmlLimits::unlimited(), 2).unwrap();
    let count = operation.root();
    let charging = operation
        .child(count, "pool", HtmlLimits::unlimited(), charging_tariffs())
        .unwrap();
    charge(&mut operation.charge_at(charging)).unwrap();
    operation.usage(count).unwrap().work(HtmlWork::Credits)
}

#[test]
fn pool_charges_step_the_work_owner_once_per_schema_credit() {
    macro_rules! same {
        ($charge:ident) => {{
            let admitted = steps(|charge| HtmlStringPool::$charge(charge));
            assert!(admitted > 0, "a pool charge admits work");
            assert_eq!(admitted, credits(|charge| HtmlStringPool::$charge(charge)));
        }};
    }
    same!(charge_root);
    same!(charge_mark);
    same!(charge_seal);
    same!(charge_disposal);
}

#[test]
fn a_change_that_is_not_work_is_a_charge_protocol_error() {
    let rejected = |change: HtmlChange| {
        let mut admission = Admission {
            global: &mut |_| Ok(()),
            step: &mut || Ok(()),
        };
        PoolCharge::new(&mut admission).charge(&[change])
    };
    assert_eq!(
        rejected(Change::Acquire(HtmlResource::AuxiliaryEntries, 1)),
        Err(DomError::HtmlChargeProtocol)
    );
    assert_eq!(
        rejected(Change::Work(HtmlWork::SemanticStep, 1)),
        Err(DomError::HtmlChargeProtocol)
    );
}

#[test]
fn a_batch_that_overflows_the_credit_sum_is_representation_exhausted() {
    let mut admission = Admission {
        global: &mut |_| Ok(()),
        step: &mut || panic!("an unrepresentable batch takes no step"),
    };
    assert_eq!(
        PoolCharge::new(&mut admission).charge(&[
            Change::Work(HtmlWork::RecordVisit, u64::MAX),
            Change::Work(HtmlWork::CopyByte, 1),
        ]),
        Err(DomError::RepresentationExhausted)
    );
}
