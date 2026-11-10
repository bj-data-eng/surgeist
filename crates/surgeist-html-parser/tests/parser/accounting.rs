//! A parse in a host's ledger operation, and the profile of a parse's own
//! operation. The expectations come from the ledger's documented behavior and
//! from the parse's own unhosted usage, which the hosted parse must equal.

use super::support::{fragment_context, limits};
#[allow(unused_imports)]
use crate::parser::support::{ReportDetail as _, SubjectCause as _};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use surgeist_html::accounting::{
    HtmlAccountingFault, HtmlAuxiliaryScope, HtmlCapScope, HtmlHostScope, HtmlLimits, HtmlResource,
    HtmlWork,
};
use surgeist_html_parser::*;
use surgeist_ledger::{
    Change, Dimension, Error, Operation, ProfileMode, Refusal, RefusalReason, Report, Scope,
    SharedOperation, Tariffs,
};

const SOURCE: &str = "<!doctype html><p class=a>one<b>two</b></p><table><tr><td>x</table>";

type Ledger = SharedOperation<HtmlWork, HtmlResource>;

fn request() -> HtmlDocumentParseRequest {
    crate::parser::support::offline_all(HtmlSource::utf8(SOURCE), limits())
}

/// A host operation with `scopes` scopes, profiled in summary mode without a
/// clock, and its root.
fn host(scopes: usize) -> (Ledger, Scope) {
    host_with(
        HtmlLimits::unlimited(),
        scopes,
        ProfileMode::Summary { timing: false },
    )
}

fn host_with(limits: HtmlLimits, scopes: usize, mode: ProfileMode) -> (Ledger, Scope) {
    let ledger = Operation::with_profile("host", limits, scopes, mode)
        .unwrap()
        .share();
    let root = ledger.root();
    (ledger, root)
}

fn report_of(ledger: &Ledger) -> Report<HtmlWork, HtmlResource> {
    ledger.snapshot().unwrap().expect("the host profiles")
}

fn scope_named(ledger: &Ledger, name: &str) -> Scope {
    report_of(ledger)
        .scopes
        .iter()
        .find(|scope| scope.name == name)
        .unwrap_or_else(|| panic!("no scope named {name}"))
        .scope
}

fn names(report: &Report<HtmlWork, HtmlResource>) -> Vec<&'static str> {
    report.scopes.iter().map(|scope| scope.name).collect()
}

fn live_total(ledger: &Ledger, scope: Scope) -> u64 {
    let usage = ledger.usage(scope).unwrap();
    HtmlResource::ALL
        .iter()
        .map(|resource| usage.live(*resource))
        .sum()
}

fn credits(usage: &HtmlParseUsage) -> u64 {
    usage.resource(HtmlParseResource::WorkCredits).current()
}

#[test]
fn a_parse_in_a_host_operation_has_the_usage_of_the_same_parse_alone() {
    let alone = request().parse().unwrap();
    let (ledger, root) = host(3);
    let hosted = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            root,
        )))
        .parse()
        .unwrap();
    assert_eq!(hosted.usage(), alone.usage());
    // The host's scope holds the parse once: the credit total, and the live
    // value and peak of every resource that is not a gauge.
    let usage = ledger.usage(root).unwrap();
    assert_eq!(usage.work(HtmlWork::Credits), credits(alone.usage()));
    for (parse, resource) in RESOURCES_THAT_ADD {
        let reading = alone.usage().resource(parse);
        assert_eq!(usage.live(resource), reading.current(), "{resource:?} live");
        assert_eq!(
            usage.peak(resource),
            reading.high_water(),
            "{resource:?} peak"
        );
    }
}

/// The parse resources whose host totals add the parse's own, with the schema
/// resource each is charged as: every resource except the gauges, which stay in
/// the scope that observed them.
const RESOURCES_THAT_ADD: [(HtmlParseResource, HtmlResource); 17] = [
    (HtmlParseResource::SourceUnits, HtmlResource::SourceUnits),
    (
        HtmlParseResource::BufferedUtf16Units,
        HtmlResource::BufferedUtf16Units,
    ),
    (
        HtmlParseResource::TokenAttributes,
        HtmlResource::TokenAttributes,
    ),
    (
        HtmlParseResource::AuxiliaryEntries,
        HtmlResource::AuxiliaryEntries,
    ),
    (HtmlParseResource::ManagedNodes, HtmlResource::ManagedNodes),
    (HtmlParseResource::ManagedEdges, HtmlResource::ManagedEdges),
    (
        HtmlParseResource::ManagedAttributes,
        HtmlResource::ManagedAttributes,
    ),
    (
        HtmlParseResource::ManagedUtf16Units,
        HtmlResource::ManagedUtf16Units,
    ),
    (
        HtmlParseResource::SourceMapEntries,
        HtmlResource::SourceMapEntries,
    ),
    (
        HtmlParseResource::SourceMapPayloadUnits,
        HtmlResource::SourceMapPayloadUnits,
    ),
    (HtmlParseResource::Diagnostics, HtmlResource::Diagnostics),
    (
        HtmlParseResource::DiagnosticPayloadUnits,
        HtmlResource::DiagnosticPayloadUnits,
    ),
    (
        HtmlParseResource::ContextEffects,
        HtmlResource::ContextEffects,
    ),
    (HtmlParseResource::OpenElements, HtmlResource::OpenElements),
    (
        HtmlParseResource::ActiveFormattingEntries,
        HtmlResource::ActiveFormattingEntries,
    ),
    (
        HtmlParseResource::ManagedCapacityBytes,
        HtmlResource::ManagedCapacityBytes,
    ),
    (
        HtmlParseResource::PeakTransientBytes,
        HtmlResource::PeakTransientBytes,
    ),
];

/// Each parser work category with the schema work dimension that carries it.
const CATEGORIES: [(HtmlParseWorkCategory, HtmlWork); 8] = [
    (
        HtmlParseWorkCategory::InputUnitRead,
        HtmlWork::InputUnitRead,
    ),
    (HtmlParseWorkCategory::InspectByte, HtmlWork::InspectByte),
    (HtmlParseWorkCategory::CopyByte, HtmlWork::CopyByte),
    (HtmlParseWorkCategory::StateStep, HtmlWork::StateStep),
    (HtmlParseWorkCategory::RecordVisit, HtmlWork::RecordVisit),
    (HtmlParseWorkCategory::RecordWrite, HtmlWork::RecordWrite),
    (HtmlParseWorkCategory::ReserveByte, HtmlWork::ReserveByte),
    (HtmlParseWorkCategory::ProviderCall, HtmlWork::ProviderCall),
];

#[test]
fn the_gauges_of_a_parse_stay_below_the_host_scope() {
    let (ledger, root) = host(3);
    let report = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            root,
        )))
        .parse()
        .unwrap();
    let usage = ledger.usage(root).unwrap();
    for gauge in [HtmlResource::TreeDepth, HtmlResource::AtomicLayoutBytes] {
        assert_eq!(usage.live(gauge), 0, "{gauge:?}");
        assert_eq!(usage.peak(gauge), 0, "{gauge:?}");
    }
    let charging = ledger
        .usage(scope_named(&ledger, "html-parse-charging"))
        .unwrap();
    assert_eq!(
        charging.peak(HtmlResource::TreeDepth),
        report
            .usage()
            .resource(HtmlParseResource::TreeDepth)
            .high_water()
    );
    assert!(charging.peak(HtmlResource::TreeDepth) > 0);
}

#[test]
fn a_credit_cap_of_the_host_scope_stops_the_parse_with_the_hosts_limit() {
    let alone = request().parse().unwrap();
    let cap = credits(alone.usage()) / 2;
    let limits = HtmlLimits::from_fn(
        |work| {
            if work == HtmlWork::Credits {
                cap
            } else {
                u64::MAX
            }
        },
        |_| u64::MAX,
    );
    let (ledger, root) = host_with(limits, 3, ProfileMode::Off);
    let Err(failure) = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            root,
        )))
        .parse()
    else {
        panic!("the host cap is below the work the parse needs");
    };
    let Some(HtmlParseStop::LimitExceeded {
        resource,
        limit,
        current,
        requested_delta,
    }) = failure.first_stop()
    else {
        panic!("a limit stop, not {:?}", failure.first_stop());
    };
    assert_eq!(resource, HtmlParseResource::WorkCredits);
    assert_eq!(limit, cap);
    // The ledger names the item that crossed the cap with the host's total
    // before it.
    assert!(current <= cap && current + requested_delta > cap);
    assert!(ledger.usage(root).unwrap().work(HtmlWork::Credits) <= cap);
}

#[test]
fn a_resource_cap_of_the_host_scope_stops_the_parse_with_the_hosts_limit() {
    let limits = HtmlLimits::from_fn(
        |_| u64::MAX,
        |resource| {
            if resource == HtmlResource::ManagedNodes {
                3
            } else {
                u64::MAX
            }
        },
    );
    let (ledger, root) = host_with(limits, 3, ProfileMode::Off);
    let Err(failure) = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            root,
        )))
        .parse()
    else {
        panic!("the host holds three nodes and the document has more");
    };
    assert_eq!(
        failure.first_stop(),
        Some(HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::ManagedNodes,
            limit: 3,
            current: 3,
            requested_delta: 1,
        })
    );
}

#[test]
fn a_parse_adds_two_scopes_and_phase_observation_adds_four() {
    let (ledger, root) = host(3);
    let _kept = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            root,
        )))
        .parse()
        .unwrap();
    assert_eq!(
        names(&report_of(&ledger)),
        ["host", "html-parse", "html-parse-charging"]
    );

    let (ledger, root) = host(7);
    let parsed = request()
        .with_accounting(
            HtmlParseAccounting::host(HtmlHostScope::new(ledger.clone(), root)).observing_phases(),
        )
        .parse()
        .unwrap();
    let report = report_of(&ledger);
    assert_eq!(
        names(&report),
        [
            "host",
            "html-parse",
            "html-parse-charging",
            "html-parse-admission",
            "html-parse-construction",
            "html-parse-finalization",
            "html-parse-publication",
        ]
    );
    let scope = |name: &str| report.scopes.iter().find(|s| s.name == name).unwrap();
    assert_eq!(scope("html-parse").parent, Some(root));
    let charging = scope("html-parse-charging");
    assert_eq!(charging.parent, Some(scope("html-parse").scope));
    for phase in [
        "html-parse-admission",
        "html-parse-construction",
        "html-parse-finalization",
        "html-parse-publication",
    ] {
        let phase = scope(phase);
        assert_eq!(phase.parent, Some(charging.scope), "{}", phase.name);
        assert_eq!(phase.phases, 1, "{} is observed once", phase.name);
        assert!(phase.accepted_items > 0, "{} receives work", phase.name);
    }
    // The phases receive the work: each category's total is the sum over the
    // phases, and the charging scope holds no work of its own.
    let all = parsed.usage();
    for (category, dimension) in CATEGORIES {
        let phases: u64 = report
            .scopes
            .iter()
            .filter(|s| s.parent == Some(charging.scope))
            .map(|s| s.usage.work(dimension))
            .sum();
        assert_eq!(phases, all.work(category), "{category:?}");
        assert_eq!(charging.usage.work(dimension), all.work(category));
    }
}

#[test]
fn a_scope_bound_too_small_for_the_parse_stops_it_with_a_typed_fault() {
    let (ledger, root) = host(2);
    let Err(failure) = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(ledger, root)))
        .parse()
    else {
        panic!("the host has room for one scope below its root");
    };
    assert_eq!(
        failure.first_stop(),
        Some(HtmlParseStop::Accounting(HtmlAccountingFault::ScopeLimit {
            limit: 2
        }))
    );
}

#[test]
fn a_scope_of_another_operation_stops_the_parse_without_a_report() {
    let (ledger, _) = host(3);
    let (_, foreign) = host(3);
    let Err(failure) = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger, foreign,
        )))
        .parse()
    else {
        panic!("a foreign scope serves no parse");
    };
    assert_eq!(
        failure.first_stop(),
        Some(HtmlParseStop::Accounting(HtmlAccountingFault::ForeignScope))
    );
    // The failure describes the accounting stop as it describes the others.
    assert_eq!(
        failure.to_string(),
        "HTML parse failed: Stop(Accounting(ForeignScope))"
    );
}

static CLOCK_READS: AtomicUsize = AtomicUsize::new(0);

fn counting_clock() -> Duration {
    CLOCK_READS.fetch_add(1, Ordering::SeqCst);
    Duration::ZERO
}

#[test]
fn profiling_off_reads_no_clock_and_timed_profiling_reads_it() {
    let host_with_clock = |mode| {
        let ledger =
            Operation::with_profile_clock("host", HtmlLimits::unlimited(), 7, mode, counting_clock)
                .unwrap()
                .share();
        let root = ledger.root();
        (ledger, root)
    };
    let before = CLOCK_READS.load(Ordering::SeqCst);
    let (ledger, root) = host_with_clock(ProfileMode::Off);
    request()
        .with_accounting(
            HtmlParseAccounting::host(HtmlHostScope::new(ledger.clone(), root)).observing_phases(),
        )
        .parse()
        .unwrap();
    assert_eq!(CLOCK_READS.load(Ordering::SeqCst), before);
    assert!(ledger.snapshot().unwrap().is_none());

    let (ledger, root) = host_with_clock(ProfileMode::Summary { timing: true });
    request()
        .with_accounting(
            HtmlParseAccounting::host(HtmlHostScope::new(ledger, root)).observing_phases(),
        )
        .parse()
        .unwrap();
    assert!(CLOCK_READS.load(Ordering::SeqCst) > before);
}

#[test]
fn a_parse_profiles_its_own_operation_when_its_request_selects_a_mode() {
    let report = request().parse().unwrap();
    assert!(report.profile().unwrap().is_none());
    let report = request()
        .with_accounting(HtmlParseAccounting::profiled(ProfileMode::Off))
        .parse()
        .unwrap();
    assert!(report.profile().unwrap().is_none());

    let report = request()
        .with_accounting(
            HtmlParseAccounting::profiled(ProfileMode::Summary { timing: false })
                .observing_phases(),
        )
        .parse()
        .unwrap();
    let profile = report.profile().unwrap().expect("a summary was selected");
    assert_eq!(
        names(&profile),
        [
            "html-parse-host",
            "html-parse",
            "html-parse-charging",
            "html-parse-admission",
            "html-parse-construction",
            "html-parse-finalization",
            "html-parse-publication",
        ]
    );
    assert_eq!(profile.elapsed, None, "no timing, no clock");
    let root = profile.scopes.first().unwrap();
    assert_eq!(root.usage.work(HtmlWork::Credits), credits(report.usage()));
    assert!(profile.scopes.iter().skip(3).all(|scope| scope.phases == 1));
}

#[test]
fn a_failed_parse_profiles_its_own_operation_too() {
    let limits = limits().into_builder().work_credits(50).finish().unwrap();
    let Err(failure) = crate::parser::support::offline_all(HtmlSource::utf8(SOURCE), limits)
        .with_accounting(
            HtmlParseAccounting::profiled(ProfileMode::Summary { timing: false })
                .observing_phases(),
        )
        .parse()
    else {
        panic!("fifty credits cannot parse the document");
    };
    let profile = failure.profile().unwrap().expect("a summary was selected");
    let first_refusal: Option<Refusal<HtmlWork, HtmlResource>> =
        profile.scopes.iter().find_map(|scope| scope.first_refusal);
    let refusal = first_refusal.expect("the cap refused a batch");
    assert!(matches!(refusal.reason, RefusalReason::Limit { limit: 50 }));
}

#[test]
fn dropping_the_report_releases_what_the_parse_still_held_in_the_host() {
    let (ledger, root) = host(3);
    let report = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            root,
        )))
        .parse()
        .unwrap();
    let charging = scope_named(&ledger, "html-parse-charging");
    assert!(
        live_total(&ledger, charging) > 0,
        "the report holds storage"
    );
    assert!(live_total(&ledger, root) > 0);
    let peak = ledger.usage(root).unwrap().peak(HtmlResource::ManagedNodes);
    drop(report);
    assert_eq!(live_total(&ledger, charging), 0);
    assert_eq!(live_total(&ledger, root), 0);
    // Peaks and work are history, not custody.
    assert_eq!(
        ledger.usage(root).unwrap().peak(HtmlResource::ManagedNodes),
        peak
    );
    assert!(ledger.usage(root).unwrap().work(HtmlWork::Credits) > 0);
}

#[test]
fn dropping_a_failure_releases_what_the_stopped_parse_still_held_in_the_host() {
    let (ledger, root) = host(3);
    let limits = limits()
        .into_builder()
        .work_credits(5_000)
        .finish()
        .unwrap();
    let Err(failure) =
        crate::parser::support::offline_all(HtmlSource::utf8(SOURCE.repeat(40)), limits)
            .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
                ledger.clone(),
                root,
            )))
            .parse()
    else {
        panic!("five thousand credits cannot parse forty copies");
    };
    let charging = scope_named(&ledger, "html-parse-charging");
    drop(failure);
    assert_eq!(live_total(&ledger, charging), 0);
    assert_eq!(live_total(&ledger, root), 0);
}

#[test]
fn a_release_the_ledger_refuses_leaves_the_custody_and_does_not_panic() {
    let (ledger, root) = host(4);
    let report = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            root,
        )))
        .parse()
        .unwrap();
    let charging = scope_named(&ledger, "html-parse-charging");
    // A scope below the parse's charging scope holds storage the charging scope
    // counts but does not hold directly, so the parse cannot release it.
    let extra = ledger
        .child(
            charging,
            "extra",
            HtmlLimits::unlimited(),
            Tariffs::identity(),
        )
        .unwrap();
    ledger
        .apply(extra, Change::Acquire(HtmlResource::OpenElements, 5))
        .unwrap();
    let before = live_total(&ledger, root);
    drop(report);
    // The whole release was refused, atomically.
    assert_eq!(live_total(&ledger, root), before);
    assert!(matches!(
        ledger.apply(charging, Change::Release(HtmlResource::OpenElements, 5)),
        Err(Error::Refused(_))
    ));
}

fn fragment_request() -> HtmlFragmentParseRequest {
    HtmlFragmentParseRequest::new(HtmlSource::utf8(SOURCE), fragment_context(), limits()).unwrap()
}

#[test]
fn a_fragment_parse_charges_a_host_operation_once_and_releases_it_on_drop() {
    let alone = fragment_request().parse().unwrap();
    let (ledger, root) = host(7);
    let hosted = fragment_request()
        .with_accounting(
            HtmlParseAccounting::host(HtmlHostScope::new(ledger.clone(), root)).observing_phases(),
        )
        .parse()
        .unwrap();
    assert_eq!(hosted.usage(), alone.usage());
    assert_eq!(
        ledger.usage(root).unwrap().work(HtmlWork::Credits),
        credits(alone.usage())
    );
    let charging = scope_named(&ledger, "html-parse-charging");
    assert!(live_total(&ledger, charging) > 0);
    drop(hosted);
    assert_eq!(live_total(&ledger, charging), 0);
    assert_eq!(live_total(&ledger, root), 0);
}

#[test]
fn a_fragment_parse_profiles_its_own_operation_when_its_request_selects_a_mode() {
    let report = fragment_request()
        .with_accounting(
            HtmlParseAccounting::profiled(ProfileMode::Summary { timing: false })
                .observing_phases(),
        )
        .parse()
        .unwrap();
    let profile = report.profile().unwrap().expect("a summary was selected");
    assert_eq!(profile.scopes.len(), 7);
    assert_eq!(
        profile
            .scopes
            .first()
            .unwrap()
            .usage
            .work(HtmlWork::Credits),
        credits(report.usage())
    );
    let failed = HtmlFragmentParseRequest::new(
        HtmlSource::utf8(SOURCE),
        fragment_context(),
        limits().into_builder().work_credits(50).finish().unwrap(),
    )
    .unwrap()
    .with_accounting(
        HtmlParseAccounting::profiled(ProfileMode::Summary { timing: false }).observing_phases(),
    )
    .parse();
    let Err(failure) = failed else {
        panic!("fifty credits cannot parse the fragment");
    };
    assert!(failure.profile().unwrap().is_some());
}

#[test]
fn the_last_accounting_a_request_names_is_the_only_one_that_applies() {
    let (ledger, root) = host(3);
    let report = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            root,
        )))
        .with_accounting(HtmlParseAccounting::profiled(ProfileMode::Summary {
            timing: false,
        }))
        .parse()
        .unwrap();
    // The host named first is not charged, and the parse profiled its own
    // operation.
    assert_eq!(ledger.usage(root).unwrap().work(HtmlWork::Credits), 0);
    assert_eq!(names(&report_of(&ledger)), ["host"]);
    assert!(report.profile().unwrap().is_some());

    let (ledger, root) = host(3);
    let report = request()
        .with_accounting(HtmlParseAccounting::profiled(ProfileMode::Summary {
            timing: false,
        }))
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            root,
        )))
        .parse()
        .unwrap();
    assert_eq!(
        ledger.usage(root).unwrap().work(HtmlWork::Credits),
        credits(report.usage())
    );
    assert!(report.profile().unwrap().is_none());
}

#[test]
fn a_profiled_request_without_phase_observation_reports_three_scopes() {
    let report = request()
        .with_accounting(HtmlParseAccounting::profiled(ProfileMode::Summary {
            timing: false,
        }))
        .parse()
        .unwrap();
    let profile = report.profile().unwrap().expect("a summary was selected");
    assert_eq!(
        names(&profile),
        ["html-parse-host", "html-parse", "html-parse-charging"]
    );
    assert_eq!(
        profile
            .scopes
            .first()
            .unwrap()
            .usage
            .work(HtmlWork::Credits),
        credits(report.usage())
    );
}

#[test]
fn phase_observation_of_a_profiled_operation_that_profiles_nothing_still_charges_it() {
    let report = request()
        .with_accounting(HtmlParseAccounting::profiled(ProfileMode::Off).observing_phases())
        .parse()
        .unwrap();
    assert!(report.profile().unwrap().is_none());
    assert_eq!(report.usage(), request().parse().unwrap().usage());
}

/// `< ` ten times with no DOCTYPE: ten tokenizer errors, one for each U+003C
/// followed by U+0020 (13.2.5.6 Tag open state), and one for the missing
/// DOCTYPE (13.2.6.4.1 "initial", "anything else").
const TEN_ERRORS: &str = "< < < < < < < < < < ";

#[test]
fn a_host_diagnostics_cap_below_the_parse_cap_drops_diagnostics_and_the_parse_completes() {
    let alone = crate::parser::support::offline_all(HtmlSource::utf8(TEN_ERRORS), limits())
        .parse()
        .unwrap();
    assert_eq!(alone.diagnostics().len(), 11);
    assert_eq!(alone.diagnostics().dropped(), 0);

    let host_limits = HtmlLimits::from_fn(
        |_| u64::MAX,
        |resource| {
            if resource == HtmlResource::Diagnostics {
                3
            } else {
                u64::MAX
            }
        },
    );
    let (ledger, root) = host_with(host_limits, 3, ProfileMode::Off);
    let hosted = crate::parser::support::offline_all(HtmlSource::utf8(TEN_ERRORS), limits())
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            root,
        )))
        .parse()
        .unwrap();
    // The host retains three of the eleven, and the parse counts the rest.
    assert_eq!(hosted.diagnostics().len(), 3);
    assert_eq!(hosted.diagnostics().dropped(), 8);
    assert_eq!(
        ledger.usage(root).unwrap().live(HtmlResource::Diagnostics),
        3
    );
}

#[test]
fn a_supplied_scope_that_is_not_the_root_holds_the_parse_and_its_ancestors_count_it() {
    let (ledger, root) = host(5);
    let tenant = ledger
        .child(root, "tenant", HtmlLimits::unlimited(), Tariffs::identity())
        .unwrap();
    let report = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            tenant,
        )))
        .parse()
        .unwrap();
    let profile = report_of(&ledger);
    assert_eq!(
        names(&profile),
        ["host", "tenant", "html-parse", "html-parse-charging"]
    );
    let scope = |name: &str| profile.scopes.iter().find(|s| s.name == name).unwrap();
    assert_eq!(scope("html-parse").parent, Some(tenant));
    for at in [tenant, root] {
        assert_eq!(
            ledger.usage(at).unwrap().work(HtmlWork::Credits),
            credits(report.usage())
        );
    }
    let charging = scope("html-parse-charging").scope;
    assert!(live_total(&ledger, tenant) > 0);
    drop(report);
    assert_eq!(live_total(&ledger, charging), 0);
    assert_eq!(live_total(&ledger, tenant), 0);
    assert_eq!(live_total(&ledger, root), 0);
}

#[test]
fn a_parse_stopped_in_any_phase_releases_its_custody_when_the_failure_drops() {
    const PHASES: [&str; 4] = [
        "html-parse-admission",
        "html-parse-construction",
        "html-parse-finalization",
        "html-parse-publication",
    ];
    // The credits each phase receives in a complete parse.
    let (ledger, root) = host(7);
    request()
        .with_accounting(
            HtmlParseAccounting::host(HtmlHostScope::new(ledger.clone(), root)).observing_phases(),
        )
        .parse()
        .unwrap();
    let profile = report_of(&ledger);
    let phase_credits: Vec<u64> = PHASES
        .iter()
        .map(|name| {
            let scope = profile.scopes.iter().find(|s| s.name == *name).unwrap();
            CATEGORIES
                .iter()
                .map(|(_, dimension)| scope.usage.work(*dimension))
                .sum()
        })
        .collect();
    assert!(phase_credits.iter().all(|credits| *credits > 1));
    let mut before = 0_u64;
    for (index, name) in PHASES.iter().enumerate() {
        // A credit cap one credit into the phase stops the parse there.
        let cap = before + 1;
        before += phase_credits[index];
        let limits = HtmlLimits::from_fn(
            |work| {
                if work == HtmlWork::Credits {
                    cap
                } else {
                    u64::MAX
                }
            },
            |_| u64::MAX,
        );
        let (ledger, root) = host_with(limits, 7, ProfileMode::Summary { timing: false });
        let Err(failure) = request()
            .with_accounting(
                HtmlParseAccounting::host(HtmlHostScope::new(ledger.clone(), root))
                    .observing_phases(),
            )
            .parse()
        else {
            panic!("a cap of {cap} credits stops the parse in {name}");
        };
        let refused = report_of(&ledger);
        let phase = refused.scopes.iter().find(|s| s.name == *name).unwrap();
        assert!(phase.refused_batches > 0, "{name} refused the batch");
        let charging = scope_named(&ledger, "html-parse-charging");
        drop(failure);
        assert_eq!(live_total(&ledger, charging), 0, "{name}");
        assert_eq!(live_total(&ledger, root), 0, "{name}");
    }
}

/// Every scope a parse creates takes its name from the schema: the cap scope from
/// `HtmlCapScope`, and the charging scope, the phase scopes and a profiled operation's root from
/// `HtmlAuxiliaryScope`.
#[test]
fn every_scope_a_parse_creates_is_named_by_the_schema() {
    let report = request()
        .with_accounting(
            HtmlParseAccounting::profiled(ProfileMode::Summary { timing: false })
                .observing_phases(),
        )
        .parse()
        .unwrap()
        .profile()
        .unwrap()
        .expect("a summary was selected");
    assert_eq!(
        names(&report),
        [
            HtmlAuxiliaryScope::ParseHost.name(),
            HtmlCapScope::Parse.name(),
            HtmlAuxiliaryScope::ParseCharging.name(),
            HtmlAuxiliaryScope::ParseAdmission.name(),
            HtmlAuxiliaryScope::ParseConstruction.name(),
            HtmlAuxiliaryScope::ParseFinalization.name(),
            HtmlAuxiliaryScope::ParsePublication.name(),
        ]
    );
    // An operation without profiling names its cap scope the same way.
    let (ledger, root) = host(3);
    let _kept = request()
        .with_accounting(HtmlParseAccounting::host(HtmlHostScope::new(
            ledger.clone(),
            root,
        )))
        .parse()
        .unwrap();
    assert_eq!(
        names(&report_of(&ledger)),
        [
            "host",
            HtmlCapScope::Parse.name(),
            HtmlAuxiliaryScope::ParseCharging.name(),
        ]
    );
}
