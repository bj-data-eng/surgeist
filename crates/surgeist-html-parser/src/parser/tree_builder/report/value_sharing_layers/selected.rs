//! One real parser promotion and its successful holder lifetimes per process.
//! Off retains both pools' fixed lifecycle and mandatory promotion traversal.
//! The native seal checkpoint precedes report promotion: it is neither a
//! both-pools-prepared checkpoint nor evidence of a terminal phase peak.

use super::*;
use std::{io, time::Duration};
use surgeist_test::checkpoint::{Config, Gate, Identity};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Case {
    EqualBanks,
    DistinctBanksText,
}

impl Case {
    fn name(self) -> &'static str {
        match self {
            Self::EqualBanks => "equal-banks",
            Self::DistinctBanksText => "distinct-banks-text",
        }
    }

    fn markup(self) -> &'static str {
        match self {
            Self::EqualBanks => EQUAL_BANKS,
            Self::DistinctBanksText => DISTINCT_BANKS_TEXT,
        }
    }

    fn literal(self, context: Context) -> String {
        if context == Context::Fragment {
            self.markup().to_owned()
        } else {
            format!("<!doctype html>{}", self.markup())
        }
    }

    /// Reuse the original literal, provenance and backing oracles. All query
    /// IDs and scratch vectors die inside these helpers; only this bank escapes.
    fn check(
        self,
        report: &ImmutableParseReport,
        mode: ValueSharingMode,
    ) -> Arc<Vec<surgeist_html::Attribute>> {
        match self {
            Self::EqualBanks => check_equal_banks(report, mode),
            Self::DistinctBanksText => check_distinct_banks_text(report, mode),
        }
    }

    fn hold_bank(self, report: &ImmutableParseReport) -> Arc<Vec<surgeist_html::Attribute>> {
        let ids = elements(
            report,
            match self {
                Self::EqualBanks => "td",
                Self::DistinctBanksText => "p",
            },
        );
        element(report, &ids[0])
            .attributes()
            .shared_attributes()
            .clone()
    }

    /// Independent literal checks require no report, source ID or NodeId.
    fn check_held_bank(self, held: &[surgeist_html::Attribute]) {
        assert_eq!(held.len(), 2);
        let expected: [(&[u16], &[u16]); 2] = match self {
            Self::EqualBanks => [
                (&[99, 108, 97, 115, 115], &[107]),
                (&[116, 105, 116, 108, 101], &[118]),
            ],
            Self::DistinctBanksText => [(&[105, 100], &[97]), (&[99, 108, 97, 115, 115], &[107])],
        };
        for (attribute, (name, value)) in held.iter().zip(expected) {
            assert!(attribute.name().namespace().uri().is_none());
            assert!(attribute.name().prefix().is_none());
            assert_eq!(attribute.name().local_name().code_units(), name);
            assert_eq!(attribute.value().code_units(), value);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Context {
    Document,
    Fragment,
}

impl Context {
    fn name(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Fragment => "fragment",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sample {
    Warmup,
    First,
    Second,
    Third,
}

impl Sample {
    fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "warmup" => Ok(Self::Warmup),
            "0" => Ok(Self::First),
            "1" => Ok(Self::Second),
            "2" => Ok(Self::Third),
            _ => Err("invalid SURGEIST_MEMORY_SAMPLE"),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Warmup => "warmup",
            Self::First => "0",
            Self::Second => "1",
            Self::Third => "2",
        }
    }
}

fn mode_name(mode: ValueSharingMode) -> &'static str {
    match mode {
        ValueSharingMode::Off => "off",
        ValueSharingMode::BankOnly => "banks-only",
        ValueSharingMode::Combined => "combined",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Selection {
    case: Case,
    context: Context,
    mode: ValueSharingMode,
    sample: Sample,
}

impl Selection {
    fn parse(
        population: &str,
        case: &str,
        context: &str,
        mode: &str,
        sample: &str,
    ) -> Result<Self, &'static str> {
        if population != "parser-layered" {
            return Err("SURGEIST_MEMORY_POPULATION must be parser-layered");
        }
        let case = match case {
            "equal-banks" => Case::EqualBanks,
            "distinct-banks-text" => Case::DistinctBanksText,
            _ => return Err("invalid SURGEIST_MEMORY_CASE"),
        };
        let context = match context {
            "document" => Context::Document,
            "fragment" => Context::Fragment,
            _ => return Err("invalid SURGEIST_MEMORY_CONTEXT"),
        };
        let mode = match mode {
            "off" => ValueSharingMode::Off,
            "banks-only" => ValueSharingMode::BankOnly,
            "combined" => ValueSharingMode::Combined,
            _ => return Err("invalid SURGEIST_MEMORY_MODE"),
        };
        Ok(Self {
            case,
            context,
            mode,
            sample: Sample::parse(sample)?,
        })
    }
}

impl Identity for Selection {
    fn record(self, token: &str, pid: u32, stage: &str, ordinal: Option<usize>) -> String {
        format!(
            "token={token}\npid={pid}\nfixture=parser-layered\ncase={}\ncontext={}\nmode={}\nsample={}\nstage={stage}\ncheckpoint={}\n",
            self.case.name(),
            self.context.name(),
            mode_name(self.mode),
            self.sample.name(),
            ordinal.unwrap_or(0)
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    HarnessOnly,
    SealedBeforeReport,
    ReportHeld,
    BankHeldAfterReport,
    Released,
}

impl Stage {
    const ALL: [Self; 5] = [
        Self::HarnessOnly,
        Self::SealedBeforeReport,
        Self::ReportHeld,
        Self::BankHeldAfterReport,
        Self::Released,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::HarnessOnly => "harness-only",
            Self::SealedBeforeReport => "sealed-before-report",
            Self::ReportHeld => "report-held",
            Self::BankHeldAfterReport => "bank-held-after-report",
            Self::Released => "released",
        }
    }
}

fn checkpoints(value: &str) -> Result<Vec<Stage>, &'static str> {
    match value {
        "none" => return Ok(Vec::new()),
        "all" => return Ok(Stage::ALL.to_vec()),
        _ => {}
    }
    let mut result = Vec::new();
    let mut previous = None;
    for name in value.split(',') {
        let index = Stage::ALL
            .iter()
            .position(|stage| stage.name() == name)
            .ok_or("invalid parser SURGEIST_MEMORY_HOLD_STAGE")?;
        if previous.is_some_and(|previous| index <= previous) {
            return Err("parser checkpoints must be distinct and in lifecycle order");
        }
        previous = Some(index);
        result.push(Stage::ALL[index]);
    }
    Ok(result)
}

fn required(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("explicit UTF-8 {name} is required"))
}

fn optional(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(value) => Some(value),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => panic!("{name} must be UTF-8"),
    }
}

fn checkpoint_config(
    enabled: bool,
    directory: Option<&str>,
    token: Option<&str>,
    timeout: Option<&str>,
) -> io::Result<Option<Config>> {
    let invalid = |message| io::Error::new(io::ErrorKind::InvalidInput, message);
    if !enabled {
        if directory.is_some() || token.is_some() || timeout.is_some() {
            return Err(invalid(
                "hold=none forbids inherited acknowledgement configuration",
            ));
        }
        return Ok(None);
    }
    let directory = directory
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid("missing SURGEIST_MEMORY_STAGE_DIR"))?;
    let token = token.ok_or_else(|| invalid("missing SURGEIST_MEMORY_TOKEN"))?;
    let seconds: u64 = timeout
        .ok_or_else(|| invalid("missing SURGEIST_MEMORY_ACK_TIMEOUT_SECONDS"))?
        .parse()
        .map_err(|_| invalid("invalid acknowledgement timeout"))?;
    Config::new(directory.into(), token.into(), Duration::from_secs(seconds)).map(Some)
}

fn configuration() -> (Selection, Vec<Stage>, Option<Config>) {
    let selection = Selection::parse(
        &required("SURGEIST_MEMORY_POPULATION"),
        &required("SURGEIST_MEMORY_CASE"),
        &required("SURGEIST_MEMORY_CONTEXT"),
        &required("SURGEIST_MEMORY_MODE"),
        &required("SURGEIST_MEMORY_SAMPLE"),
    )
    .expect("explicit fixed parser layer selection");
    let stages = checkpoints(&required("SURGEIST_MEMORY_HOLD_STAGE"))
        .expect("explicit parser checkpoint selection");
    let directory = optional("SURGEIST_MEMORY_STAGE_DIR");
    let token = optional("SURGEIST_MEMORY_TOKEN");
    let timeout = optional("SURGEIST_MEMORY_ACK_TIMEOUT_SECONDS");
    let config = checkpoint_config(
        !stages.is_empty(),
        directory.as_deref(),
        token.as_deref(),
        timeout.as_deref(),
    )
    .expect("explicit parser stage gate configuration");
    (selection, stages, config)
}

fn run_sample(selection: Selection, observe: &mut impl FnMut(Stage)) {
    observe(Stage::HarnessOnly);
    let mut sealed = {
        let literal = selection.case.literal(selection.context);
        let sealed = Sealed::new(
            selection.context == Context::Fragment,
            selection.mode,
            &literal,
        );
        assert_eq!(sealed.source().as_utf8(), Some(literal.as_str()));
        sealed
    }; // The extra authored String is gone before the native seal hold.
    observe(Stage::SealedBeforeReport);
    let report = {
        let source = sealed.source().identity().clone();
        let capture = match &sealed {
            Sealed::Fragment(Some(value)) => Some(value.context().capture().identity().clone()),
            _ => None,
        };
        let report = sealed
            .report()
            .expect("successful actual terminal promotion");
        assert_eq!(report.source().identity(), &source);
        if let Some(capture) = capture {
            assert!(report.document().is_none());
            assert_eq!(report.context().unwrap().capture().identity(), &capture);
        } else {
            assert!(report.fragment().is_none());
            assert_eq!(
                report.document().unwrap().mode(),
                HtmlDocumentMode::NoQuirks
            );
        }
        report
    }; // Extra source and context identity handles are gone before report-held.
    assert!(matches!(
        sealed,
        Sealed::Document(None) | Sealed::Fragment(None)
    ));
    drop(sealed);
    assert_eq!(
        report.source().as_utf8(),
        Some(selection.case.literal(selection.context).as_str())
    );
    drop(selection.case.check(&report, selection.mode));
    let work = report.usage().resource(HtmlParseResource::WorkCredits);
    eprintln!(
        "memory_parser_layer_admission retained_construction_bytes={} work_current={} work_high_water={} logical_not_physical=true source_and_report_header_excluded_from_retained_construction=true",
        report.retained_construction_bytes(),
        work.current(),
        work.high_water()
    );
    observe(Stage::ReportHeld);
    // No query IDs or extra bank alias survive the previous hold. This one
    // actual bank becomes the sole intentional semantic holder after report drop.
    let held = selection.case.hold_bank(&report);
    let weak = Arc::downgrade(&held);
    drop(report);
    selection.case.check_held_bank(&held);
    assert!(weak.upgrade().is_some());
    observe(Stage::BankHeldAfterReport);
    drop(held);
    assert!(
        weak.upgrade().is_none(),
        "closed terminal pools retain no strong bank"
    );
    // The diagnostic Weak deliberately remains visible during bank-held, then
    // dies before released. No allocator reclamation is inferred from upgrade.
    drop(weak);
    observe(Stage::Released);
}

#[test]
#[ignore = "explicit fresh-process release parser fixture; select one context/case/mode/sample and optional ordered checkpoints"]
fn selected_parser_layer_holder_stages() {
    let profile = if cfg!(debug_assertions) {
        Err("select this fixture in a release build")
    } else {
        Ok(())
    };
    profile.expect("release parser fixture required");
    let (selection, stages, config) = configuration();
    let names = stages.iter().map(|stage| stage.name()).collect();
    drop(stages);
    let mut gate = config
        .map(|config| Gate::open(config, names))
        .transpose()
        .expect("fresh runner-owned parser stage directory");
    let pid = std::process::id();
    eprintln!(
        "memory_parser_layer_selected profile=release pid={pid} case={} context={} mode={} sample={} physical_backend=external_not_run_by_fixture timings_not_reported=true common_fixed_pool_lifecycle_and_promotion=true off_is_not_pool_free=true sealed_hold_is_not_both_prepared=true terminal_peak_not_established=true",
        selection.case.name(),
        selection.context.name(),
        mode_name(selection.mode),
        selection.sample.name()
    );
    let mut observe = |stage: Stage| {
        eprintln!(
            "memory_parser_layer_stage pid={pid} case={} context={} mode={} sample={} stage={} native_seal_live={} report_live={} held_bank_live={} bank_weak_live={} extra_source_context_ids_live=false node_id_scratch_live=false transport_storage_not_subtracted=true allocator_reclamation_not_observed=true",
            selection.case.name(),
            selection.context.name(),
            mode_name(selection.mode),
            selection.sample.name(),
            stage.name(),
            stage == Stage::SealedBeforeReport,
            stage == Stage::ReportHeld,
            stage == Stage::BankHeldAfterReport,
            stage == Stage::BankHeldAfterReport,
        );
        if let Some(gate) = &mut gate {
            gate.checkpoint(selection, pid, stage.name())
                .unwrap_or_else(|error| panic!("parser stage {} failed: {error}", stage.name()));
        }
    };
    run_sample(selection, &mut observe);
    if let Some(gate) = gate {
        assert!(
            gate.is_complete(),
            "all selected parser stages acknowledged"
        );
    }
    eprintln!("memory_parser_layer_complete pid={pid} physical_total_not_inferred=true");
}

mod tests;
