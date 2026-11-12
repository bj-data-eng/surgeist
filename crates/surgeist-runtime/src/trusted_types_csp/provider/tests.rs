use super::*;
use crate::trusted_types_csp::Decision;
use std::{cell::Cell, convert::Infallible};

fn allow(_: HtmlCspLayout) -> Result<(), Infallible> {
    Ok(())
}

fn step() -> Result<(), Infallible> {
    Ok(())
}

fn processor() -> PolicyProcessor {
    PolicyProcessor::new_with(&mut allow, &mut |_| unreachable!()).unwrap()
}

#[test]
fn response_lists_keep_group_order_and_skip_only_empty_header_members() {
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let mut processor = processor();
    let enforcing = [
        " ; é ; ".into(),
        "script-src 'self'".into(),
        "require-trusted-types-for 'script'; report-to enforce".into(),
    ];
    let report_only = [
        "require-trusted-types-for 'script'; report-to report".into(),
        "".into(),
    ];
    let response = processor
        .response_with(
            &mut owner,
            &enforcing,
            &report_only,
            &mut allow,
            &mut |_| unreachable!(),
        )
        .unwrap();
    let policies = processor
        .document_policies_with(DomDocumentCspState::Associated(&response), &mut allow)
        .unwrap()
        .unwrap();
    let actual: Vec<_> = policies
        .iter()
        .map(|policy| (policy.delivery(), policy.serialized().to_utf8().unwrap()))
        .collect();
    assert_eq!(
        actual,
        [
            (
                PolicyDelivery::EnforcingHeader,
                "script-src 'self'".to_owned()
            ),
            (
                PolicyDelivery::EnforcingHeader,
                "require-trusted-types-for 'script'; report-to enforce".to_owned()
            ),
            (
                PolicyDelivery::ReportOnlyHeader,
                "require-trusted-types-for 'script'; report-to report".to_owned()
            ),
        ]
    );
    let mut decision = Decision::script_mismatch(policies);
    assert_eq!(
        decision
            .next_violation(&mut step)
            .unwrap()
            .unwrap()
            .policy_index(),
        1
    );
    assert_eq!(
        decision
            .next_violation(&mut step)
            .unwrap()
            .unwrap()
            .policy_index(),
        2
    );
    assert_eq!(decision.blocked(), Some(true));
}

#[test]
fn qualified_meta_appends_even_an_empty_effective_set_without_rewriting_response() {
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let mut processor = processor();
    let response = processor
        .response_with(
            &mut owner,
            &["trusted-types *".into()],
            &["trusted-types 'none'".into()],
            &mut allow,
            &mut |_| unreachable!(),
        )
        .unwrap();
    let mut current = response.clone();
    let first = owner
        .request_meta_insertion_with(
            &current,
            processor.profile(),
            "report-uri ignored; sandbox".into(),
            &mut allow,
        )
        .unwrap();
    let HtmlMetaCspOutcome::Completed(prepared) = processor
        .prepare(&first, &mut allow, &mut |_| unreachable!())
        .unwrap()
    else {
        panic!("reached meta insertion must complete");
    };
    drop(
        owner
            .prepare_publication_with(&mut current, first, prepared, &mut allow)
            .unwrap()
            .publish(),
    );
    let second = owner
        .request_meta_insertion_with(
            &current,
            processor.profile(),
            "require-trusted-types-for 'script'".into(),
            &mut allow,
        )
        .unwrap();
    let HtmlMetaCspOutcome::Completed(prepared) = processor
        .prepare(&second, &mut allow, &mut |_| unreachable!())
        .unwrap()
    else {
        panic!("second reached meta insertion must complete");
    };
    drop(
        owner
            .prepare_publication_with(&mut current, second, prepared, &mut allow)
            .unwrap()
            .publish(),
    );
    let previous = processor
        .document_policies_with(DomDocumentCspState::Associated(&response), &mut allow)
        .unwrap()
        .unwrap();
    let next = processor
        .document_policies_with(DomDocumentCspState::Associated(&current), &mut allow)
        .unwrap()
        .unwrap();
    assert_eq!(previous.len(), 2);
    assert_eq!(
        next.iter().map(Policy::delivery).collect::<Vec<_>>(),
        [
            PolicyDelivery::EnforcingHeader,
            PolicyDelivery::ReportOnlyHeader,
            PolicyDelivery::Meta,
            PolicyDelivery::Meta
        ]
    );
    assert!(next[2].directives().next_with(&mut step).unwrap().is_none());
    let mut decision = Decision::script_mismatch(next);
    assert_eq!(
        decision
            .next_violation(&mut step)
            .unwrap()
            .unwrap()
            .policy_index(),
        3
    );
    assert_eq!(decision.blocked(), Some(true));
}

#[test]
fn current_policy_read_distinguishes_unavailable_and_rejects_a_foreign_profile() {
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let mut selected = processor();
    let other = processor();
    let delivered = selected
        .response_with(&mut owner, &[], &[], &mut allow, &mut |_| unreachable!())
        .unwrap();
    assert!(
        selected
            .document_policies_with(DomDocumentCspState::Unavailable, &mut allow)
            .unwrap()
            .is_none()
    );
    assert!(
        selected
            .document_policies_with(DomDocumentCspState::ConstructorInitialEmpty, &mut allow)
            .unwrap()
            .unwrap()
            .is_empty()
    );
    assert!(
        selected
            .document_policies_with(DomDocumentCspState::Associated(&delivered), &mut allow)
            .unwrap()
            .unwrap()
            .is_empty()
    );
    assert!(matches!(
        other.document_policies_with(DomDocumentCspState::Associated(&delivered), &mut allow),
        Err(HtmlCspError::Protocol(error)) if error.is_wrong_processor()
    ));
}

#[derive(Default)]
struct Custody {
    used: Cell<[usize; 3]>,
    calls: Cell<usize>,
    stop_at: usize,
}

impl Custody {
    fn admit(&self, layout: HtmlCspLayout) -> Result<(), &'static str> {
        let calls = self.calls.get() + 1;
        self.calls.set(calls);
        if calls == self.stop_at {
            return Err("original stop");
        }
        let before = self.used.get();
        let retained = measure(layout.retained().storage());
        let transferred = measure(layout.transferred().storage());
        self.used.set(std::array::from_fn(|index| {
            before[index] - transferred[index] + retained[index]
        }));
        Ok(())
    }

    fn release(&self, storage: HtmlCspStorage) {
        let before = self.used.get();
        let released = measure(storage);
        self.used
            .set(std::array::from_fn(|index| before[index] - released[index]));
    }
}

fn measure(storage: HtmlCspStorage) -> [usize; 3] {
    [storage.bytes(), storage.utf16_units(), storage.entries()]
}

#[test]
fn response_refusal_at_any_admission_leaves_no_draft_custody() {
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let mut processor = processor();
    let header = ["script-src 'self'".into(), "trusted-types first".into()];
    let accepted = Custody::default();
    let response = processor
        .response_with(
            &mut owner,
            &header,
            &[],
            &mut |l| accepted.admit(l),
            &mut |s| accepted.release(s),
        )
        .unwrap();
    assert_eq!(
        accepted.used.get(),
        measure(response.retained_layout().unwrap().storage())
    );
    for stop_at in 1..=accepted.calls.get() {
        let rejected = Custody {
            stop_at,
            ..Custody::default()
        };
        let error = processor
            .response_with(
                &mut owner,
                &header,
                &[],
                &mut |l| rejected.admit(l),
                &mut |s| rejected.release(s),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            PolicyListError::Csp(HtmlCspError::Interrupted("original stop"))
        ));
        assert_eq!(rejected.calls.get(), stop_at);
        assert_eq!(rejected.used.get(), [0; 3], "stop {stop_at}");
    }
    let policies = processor
        .document_policies_with(DomDocumentCspState::Associated(&response), &mut allow)
        .unwrap()
        .unwrap();
    assert_eq!(policies.len(), 2);
}

#[test]
fn a_stopped_profile_admission_releases_the_issuer() {
    let rejected = Custody {
        stop_at: 2,
        ..Custody::default()
    };
    assert!(matches!(
        PolicyProcessor::new_with(&mut |l| rejected.admit(l), &mut |s| rejected.release(s)),
        Err(HtmlCspError::Interrupted("original stop"))
    ));
    assert_eq!(rejected.used.get(), [0; 3]);
}
