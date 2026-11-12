use super::*;
use std::convert::Infallible;

fn step() -> Result<(), Infallible> {
    Ok(())
}

fn parse(serialized: &str, delivery: PolicyDelivery) -> Policy {
    Policy::parse(serialized.into(), delivery, &mut step).unwrap()
}

#[test]
fn script_requirements_use_first_valid_ascii_directive_and_keep_disposition() {
    use PolicyDelivery::{EnforcingHeader, Meta, ReportOnlyHeader};
    let nonrequiring = [
        parse(
            "require-trusted-types-for 'unknown'; require-trusted-types-for 'script'",
            EnforcingHeader,
        ),
        parse("require-trusted-types-for script 'scripting'", Meta),
        parse("require-trusted-types-for\u{000b}'script'", EnforcingHeader),
    ];
    assert!(!requires_script_values(&nonrequiring, true, &mut step).unwrap());

    let reporting = [parse(
        "\tREQUIRE-TRUSTED-TYPES-FOR 'SCRIPT'\r\n",
        ReportOnlyHeader,
    )];
    assert!(requires_script_values(&reporting, true, &mut step).unwrap());
    assert!(!requires_script_values(&reporting, false, &mut step).unwrap());

    let enforcing = [parse(
        "require-trusted-types-for 'script' é; require-trusted-types-for 'script'",
        Meta,
    )];
    assert!(requires_script_values(&enforcing, false, &mut step).unwrap());
}

#[test]
fn policy_names_use_exact_tokens_and_apply_each_allowlist_independently() {
    use PolicyDelivery::{EnforcingHeader, ReportOnlyHeader};
    let cases = [
        ("", "anything", false, false),
        ("trusted-types", "anything", false, true),
        ("trusted-types 'none'", "none", false, true),
        ("trusted-types 'none' first", "first", false, false),
        ("trusted-types First", "first", false, true),
        ("trusted-types name-#=_/@.%", "name-#=_/@.%", false, false),
        ("trusted-types 'first'", "'first'", false, true),
        ("trusted-types *", "first", true, true),
        ("trusted-types * 'ALLOW-DUPLICATES'", "first", true, false),
        (
            "trusted-types other; TRUSTED-TYPES first",
            "first",
            false,
            true,
        ),
        (
            "trusted-types\nfirst  other; report-to endpoints",
            "other",
            false,
            false,
        ),
    ];
    for (serialized, name, duplicate, blocked) in cases {
        let list = [parse(serialized, EnforcingHeader)];
        let name: Vec<_> = name.encode_utf16().collect();
        let mut decision = Decision::policy_creation(&list, &name, duplicate);
        assert!(decision.blocked().is_none());
        let violation = decision.next_violation(&mut step).unwrap();
        assert_eq!(violation.is_some(), blocked, "{serialized}");
        assert_eq!(decision.blocked(), Some(blocked));
    }
    let list = [
        parse("trusted-types *", EnforcingHeader),
        parse("trusted-types 'none'", ReportOnlyHeader),
    ];
    let mut decision = Decision::policy_creation(&list, &[0xd800, 0], false);
    let violation = decision.next_violation(&mut step).unwrap().unwrap();
    assert_eq!(violation.policy_index(), 1);
    assert_eq!(violation.directive(), Directive::TrustedTypes);
    assert_eq!(decision.blocked(), Some(false));
}

#[test]
fn mismatches_retain_every_ordered_violation_across_refusal_and_blocking() {
    use PolicyDelivery::{EnforcingHeader, Meta, ReportOnlyHeader};
    let reporting = "require-trusted-types-for 'script'; report-to first";
    let policies = [
        parse(reporting, ReportOnlyHeader),
        parse("trusted-types 'none'", EnforcingHeader),
        parse("require-trusted-types-for 'script'", Meta),
        parse(
            "require-trusted-types-for 'script'; report-to last",
            ReportOnlyHeader,
        ),
    ];
    let mut decision = Decision::script_mismatch(&policies);
    let first = decision.next_violation(&mut step).unwrap().unwrap();
    assert_eq!(first.policy_index(), 0);
    assert_eq!(first.directive(), Directive::RequireTrustedTypesFor);
    assert_eq!(first.policy().delivery(), ReportOnlyHeader);
    assert_eq!(first.policy().serialized(), &reporting.into());
    let mut work = 1_u8;
    assert!(
        decision
            .next_violation(&mut || {
                work = work.checked_sub(1_u8).ok_or("work")?;
                Ok::<(), &str>(())
            })
            .is_err()
    );
    assert!(decision.blocked().is_none());
    let enforcing = decision.next_violation(&mut step).unwrap().unwrap();
    assert_eq!(enforcing.policy_index(), 2);
    assert_eq!(enforcing.policy().delivery(), Meta);
    assert!(decision.blocked().is_none());
    let last = decision.next_violation(&mut step).unwrap().unwrap();
    assert_eq!(last.policy_index(), 3);
    assert_eq!(decision.blocked(), Some(true));
    assert!(decision.next_violation(&mut step).unwrap().is_none());
}

#[test]
fn source_inspection_and_name_comparison_can_refuse_before_advancing() {
    let text = "trusted-types duplicate 'allow-duplicates'";
    let mut steps = 0_u64;
    let policy = Policy::parse(text.into(), PolicyDelivery::EnforcingHeader, &mut || {
        steps += 1;
        Ok::<_, Infallible>(())
    })
    .unwrap();
    let mut allowance = steps - 1;
    assert!(
        Policy::parse(text.into(), PolicyDelivery::EnforcingHeader, &mut || {
            allowance = allowance.checked_sub(1_u64).ok_or(())?;
            Ok::<(), ()>(())
        })
        .is_err()
    );
    let policies = [policy];
    let name: Vec<_> = "duplicate".encode_utf16().collect();
    let mut decision = Decision::policy_creation(&policies, &name, true);
    let mut allowance = 4_u64;
    assert!(
        decision
            .next_violation(&mut || {
                allowance = allowance.checked_sub(1).ok_or(())?;
                Ok::<(), ()>(())
            })
            .is_err()
    );
    assert!(decision.blocked().is_none());
    assert!(decision.next_violation(&mut step).unwrap().is_none());
    assert_eq!(decision.blocked(), Some(false));
}
