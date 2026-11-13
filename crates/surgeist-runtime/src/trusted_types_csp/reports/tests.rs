use super::*;
use crate::trusted_types_csp::PolicyFactory;
use std::cell::Cell;
use surgeist_html::values::DocumentString;

fn step() -> Result<(), &'static str> {
    Ok(())
}

fn parse(serialized: &str, delivery: PolicyDelivery) -> Policy {
    Policy::parse(serialized.into(), delivery, &mut step).unwrap()
}

#[test]
fn meta_report_serializes_effective_directives_and_clips_exact_utf16() {
    let policies = [parse(
        " ; SCRIPT-src  'self'; REPORT-uri /old; Sandbox; require-trusted-types-for 'script'; REQUIRE-TRUSTED-TYPES-FOR other; report-to main",
        PolicyDelivery::Meta,
    )];
    let mut source = [u16::from(b'x'); 41];
    source[39] = 0xd83d;
    source[40] = 0xde03;
    let attribute = Sample::Attribute(AttributeReport::new(
        DomSourceAttributeSink::HtmlIframeSrcdoc,
        &source,
    ));
    let mut decision = Decision::script_mismatch(&policies);
    let report = decision
        .next_report(attribute, &mut step, Ok)
        .unwrap()
        .unwrap();
    let policy: Vec<_> = "script-src 'self'; require-trusted-types-for 'script'; report-to main"
        .encode_utf16()
        .collect();
    let sample: Vec<_> = "HTMLIFrameElement srcdoc|"
        .encode_utf16()
        .chain(source[..40].iter().copied())
        .collect();
    assert_eq!(report.policy_index(), 0);
    assert_eq!(report.delivery(), PolicyDelivery::Meta);
    assert_eq!(report.directive(), Directive::RequireTrustedTypesFor);
    assert_eq!(report.resource(), "trusted-types-sink");
    assert_eq!(report.original_policy(), policy);
    assert_eq!(report.sample(), sample);
    assert_eq!(report._backing, 2 * (policy.len() + sample.len()));
    assert_eq!(decision.blocked(), Some(true));
}

#[test]
fn interruption_releases_backing_and_leaves_reports_in_order() {
    struct Backing<'a>(&'a Cell<bool>);

    impl Drop for Backing<'_> {
        fn drop(&mut self) {
            self.0.set(false);
        }
    }

    let policies = [
        parse(
            "require-trusted-types-for 'script'",
            PolicyDelivery::ReportOnlyHeader,
        ),
        parse(
            "require-trusted-types-for 'script'",
            PolicyDelivery::EnforcingHeader,
        ),
    ];
    let attribute = Sample::Attribute(AttributeReport::new(
        DomSourceAttributeSink::EventHandler("onclick"),
        &[0xd800],
    ));
    let mut decision = Decision::script_mismatch(&policies);
    let refused = decision.next_report::<_, ()>(attribute, &mut step, |_| Err("storage"));
    assert_eq!(refused.unwrap_err(), ReportError::Stopped("storage"));
    assert_eq!(decision.blocked(), None);

    let allocated = Cell::new(false);
    let refused = decision.next_report(
        attribute,
        &mut || if allocated.get() { Err("work") } else { Ok(()) },
        |_| {
            allocated.set(true);
            Ok(Backing(&allocated))
        },
    );
    assert!(matches!(refused, Err(ReportError::Stopped("work"))));
    assert!(!allocated.get());
    assert_eq!(decision.blocked(), None);

    let expected: Vec<_> = "Element onclick|".encode_utf16().chain([0xd800]).collect();
    for index in 0..2 {
        let report = decision
            .next_report(attribute, &mut step, |_| Ok(()))
            .unwrap()
            .unwrap();
        assert_eq!(report.policy_index(), index);
        assert_eq!(report.sample(), expected);
    }
    assert_eq!(decision.blocked(), Some(true));
    assert!(
        decision
            .next_report(attribute, &mut step, |_| Ok(()))
            .unwrap()
            .is_none()
    );
}

#[test]
fn factory_report_uses_the_original_policy_name() {
    let policies = [parse(
        "trusted-types 'none'",
        PolicyDelivery::ReportOnlyHeader,
    )];
    let mut factory = PolicyFactory::<(), ()>::default();
    let mut name = vec![0xd800];
    name.extend([u16::from(b'n'); 40]);
    let name = DocumentString::from_utf16(name);
    let mut creation = factory.begin(&name, &policies, &mut step).unwrap();
    let report = creation
        .next_report(&mut step, |_| Ok(()))
        .unwrap()
        .unwrap();
    assert_eq!(report.directive(), Directive::TrustedTypes);
    assert_eq!(report.resource(), "trusted-types-policy");
    assert_eq!(report.sample(), &name.code_units()[..40]);
    assert!(creation.admit(|_| Ok::<_, &'static str>(())).is_ok());
}
