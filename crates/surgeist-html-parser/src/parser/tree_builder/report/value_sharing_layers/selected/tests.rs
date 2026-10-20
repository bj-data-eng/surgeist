use super::*;

#[test]
fn explicit_selection_and_stage_plan_reject_other_values_before_io() {
    for context in ["document", "fragment"] {
        for case in ["equal-banks", "distinct-banks-text"] {
            for mode in ["off", "banks-only", "combined"] {
                assert!(Selection::parse("parser-layered", case, context, mode, "0").is_ok());
            }
        }
    }
    for (population, case, context, mode, sample) in [
        ("parsed-table", "equal-banks", "document", "off", "0"),
        ("parser-layered", "other", "document", "off", "0"),
        ("parser-layered", "equal-banks", "other", "off", "0"),
        ("parser-layered", "equal-banks", "document", "other", "0"),
        ("parser-layered", "equal-banks", "document", "off", "3"),
    ] {
        assert!(Selection::parse(population, case, context, mode, sample).is_err());
    }
    assert!(checkpoints("none").unwrap().is_empty());
    assert_eq!(checkpoints("all").unwrap().len(), 5);
    assert_eq!(
        checkpoints("sealed-before-report,released").unwrap(),
        [Stage::SealedBeforeReport, Stage::Released]
    );
    for value in [
        "",
        "other",
        "report-held,report-held",
        "released,harness-only",
    ] {
        assert!(checkpoints(value).is_err());
    }
    assert!(
        checkpoint_config(false, None, None, None)
            .unwrap()
            .is_none()
    );
    assert_eq!(
        checkpoint_config(false, Some("not-accessed"), None, None)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    for (directory, token, timeout) in [
        (None, Some("token"), Some("1")),
        (Some("not-accessed"), None, Some("1")),
        (Some("not-accessed"), Some("bad\ntoken"), Some("1")),
        (Some("not-accessed"), Some("token"), Some("0")),
        (Some("not-accessed"), Some("token"), Some("301")),
    ] {
        assert_eq!(
            checkpoint_config(true, directory, token, timeout)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
    }
}

#[test]
fn checkpoint_identity_binds_the_complete_single_process_selection() {
    let selection = Selection::parse(
        "parser-layered",
        "distinct-banks-text",
        "fragment",
        "banks-only",
        "1",
    )
    .unwrap();
    assert_eq!(
        selection.record("unit-token", 42, "bank-held-after-report", Some(3)),
        "token=unit-token\npid=42\nfixture=parser-layered\ncase=distinct-banks-text\ncontext=fragment\nmode=banks-only\nsample=1\nstage=bank-held-after-report\ncheckpoint=3\n"
    );
    let first =
        Selection::parse("parser-layered", "equal-banks", "document", "off", "warmup").unwrap();
    assert_eq!(
        first.record("other-token", 91, "released", None),
        "token=other-token\npid=91\nfixture=parser-layered\ncase=equal-banks\ncontext=document\nmode=off\nsample=warmup\nstage=released\ncheckpoint=0\n"
    );
}

#[test]
fn successful_sealed_layers_release_reports_before_bank_and_weak_holders() {
    for context in [Context::Document, Context::Fragment] {
        for case in [Case::EqualBanks, Case::DistinctBanksText] {
            for mode in MODES {
                let selection = Selection {
                    case,
                    context,
                    mode,
                    sample: Sample::First,
                };
                let mut stages = Vec::new();
                run_sample(selection, &mut |stage| stages.push(stage));
                assert_eq!(
                    stages,
                    [
                        Stage::HarnessOnly,
                        Stage::SealedBeforeReport,
                        Stage::ReportHeld,
                        Stage::BankHeldAfterReport,
                        Stage::Released
                    ]
                );
            }
        }
    }
}
