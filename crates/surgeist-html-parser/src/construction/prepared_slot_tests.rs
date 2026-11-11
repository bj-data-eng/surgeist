//! Construction admits one auxiliary entry for every registry row before the
//! row exists. A parse that stops between that admission and the row's append
//! must still leave nothing retained once cleanup has run, because cleanup
//! releases entries only for the rows that exist.
//!
//! Each test parses one document that reaches a family of registry rows, and
//! counts the payload slots it prepares. It then parses the document again
//! once per prepared slot and stops the parse at the first checkpoint after
//! that slot is prepared, which is the first refusal of any step between the
//! slot's admission and its row. Every such stop must leave every retained
//! resource at zero after cleanup.
use super::tests::storage_zero;
use super::{FieldRef, KindList, RetainedPayload, literal_string};
use crate::construction::effects::tests::{graph, request};
use crate::parser::control::{
    PreparedSlotStopScope, prepared_slot_sites, prepared_slots, stop_after_prepared_slot,
};
use crate::parser::tree_builder::tests::limits;
use crate::{HtmlParseBaseFacts, HtmlParseFailure, HtmlParseLimits, HtmlParseResource, HtmlSource};
use std::convert::Infallible;
use surgeist_html::parsing::HtmlParserScriptingMode;
use surgeist_html::{
    construction::HtmlDocumentMode,
    security::csp::{
        HtmlCspError, HtmlCspLayout, HtmlCspProcessorAuthority, HtmlCspProcessorProfile,
        HtmlCspStorage, HtmlMetaCspOutcome, HtmlMetaCspProcessor, HtmlMetaCspRequest,
    },
    values::{Namespace, OrderedAttributes, QualifiedName},
};
use surgeist_html_semantics::*;

/// Resources a stopped parse returns whatever it was doing and that no kept
/// diagnostic holds. With the resources compared with the kept diagnostics in
/// `note_retained`, they are every resource cleanup refunds. `ManagedNodes` and
/// `WorkCredits` are never refunded, `SourceUnits` is admitted once for the
/// source, and `TreeDepth` and `AtomicLayoutBytes` record a high water.
const RETAINED: [HtmlParseResource; 9] = [
    HtmlParseResource::BufferedUtf16Units,
    HtmlParseResource::TokenAttributes,
    HtmlParseResource::AuxiliaryEntries,
    HtmlParseResource::ManagedEdges,
    HtmlParseResource::ManagedAttributes,
    HtmlParseResource::ManagedUtf16Units,
    HtmlParseResource::SourceMapPayloadUnits,
    HtmlParseResource::ContextEffects,
    HtmlParseResource::OpenElements,
];
/// What a stopped parse left once cleanup ran.
#[derive(Debug)]
struct Stopped {
    stop: Option<crate::HtmlParseStop>,
    cleanup_refused: bool,
    at_stop: Option<crate::HtmlParseUsage>,
    after_cleanup: Option<crate::HtmlParseUsage>,
    /// The diagnostics the failure keeps, and the distinct locations they name.
    kept_records: usize,
    kept_locations: usize,
}
fn stopped(failure: &HtmlParseFailure) -> Box<Stopped> {
    let mut locations = std::collections::HashSet::new();
    let mut kept_records = 0;
    if let Some(view) = failure.diagnostics() {
        kept_records = view.len();
        for index in 0..view.len() {
            let (_, location) = view
                .retained_record(index)
                .expect("a kept diagnostic is readable")
                .expect("a kept diagnostic is within the view");
            locations.insert(std::ptr::from_ref(location) as usize);
        }
    }
    Box::new(Stopped {
        stop: failure.first_stop(),
        cleanup_refused: failure.cleanup_error().is_some(),
        at_stop: failure.at_stop().cloned(),
        after_cleanup: failure.after_cleanup().cloned(),
        kept_records,
        kept_locations: locations.len(),
    })
}

/// Notes in `retained` what the stopped parse `stop` left, under `label`.
///
/// A stopped parse keeps the diagnostics it committed, in a records vector
/// and a locations vector that cleanup does not touch. So the expected state
/// is not zero everywhere: the diagnostics themselves are the baseline. Each
/// kept record is one `Diagnostics` unit and `size_of::<DiagnosticRecord>()`
/// bytes of capacity, and each distinct location it names is one
/// `SourceMapEntries` unit and `size_of::<DiagnosticLocation>()` bytes, held as
/// both `ManagedCapacityBytes` and `PeakTransientBytes`. Both
/// vectors hold exactly the length they were filled to. Anything beyond that
/// baseline is a leak, including capacity bytes while diagnostics are kept.
fn note_retained(retained: &mut Vec<String>, label: &str, stop: &Stopped) {
    use crate::construction::evidence::{HtmlParseDiagnostic, HtmlParseDiagnosticLocation};
    if stop.cleanup_refused {
        retained.push(format!("{label}: cleanup refused"));
    }
    let after = stop
        .after_cleanup
        .as_ref()
        .expect("a stopped parse reports cleanup");
    let expected = |resource| match resource {
        HtmlParseResource::Diagnostics => stop.kept_records as u64,
        HtmlParseResource::SourceMapEntries => stop.kept_locations as u64,
        // See the payload check below.
        HtmlParseResource::DiagnosticPayloadUnits => 0,
        HtmlParseResource::ManagedCapacityBytes | HtmlParseResource::PeakTransientBytes => {
            (stop.kept_records * std::mem::size_of::<HtmlParseDiagnostic>()
                + stop.kept_locations * std::mem::size_of::<HtmlParseDiagnosticLocation>())
                as u64
        }
        _ => 0,
    };
    // The failure view reports no payload for a kept diagnostic: a record
    // names its kind, context, severity, rule and location, and a location a
    // source range, role and token, none of them text. What the diagnostics
    // keep can never exceed the most the parse held, so a parse that never
    // held any diagnostic payload keeps none. A parse that did has a baseline
    // this test does not derive, and says so rather than expect zero.
    let at_stop = stop
        .at_stop
        .as_ref()
        .expect("a stopped parse reports its usage");
    let payload_high_water = at_stop
        .resource(HtmlParseResource::DiagnosticPayloadUnits)
        .high_water();
    if payload_high_water != 0 {
        retained.push(format!(
            "{label}: diagnostic payload reached {payload_high_water}, so its baseline is not derived"
        ));
    }
    let checked = RETAINED.into_iter().chain([
        HtmlParseResource::Diagnostics,
        HtmlParseResource::DiagnosticPayloadUnits,
        HtmlParseResource::SourceMapEntries,
        HtmlParseResource::ManagedCapacityBytes,
        HtmlParseResource::PeakTransientBytes,
    ]);
    for resource in checked {
        let current = after.resource(resource).current();
        let expected = expected(resource);
        if current != expected {
            retained.push(format!(
                "{label}: {resource:?}={current}, expected {expected}"
            ));
        }
    }
}

/// Parses once, then once more per payload slot that parse prepared, stopping
/// each at the first checkpoint after its slot. Each stop is repeated as a
/// limit of the work, capacity and entries the parse had used by then, so the
/// next step to charge any of them is refused. Returns what any stop left.
fn retained_after_each_prepared_slot(
    parse: &dyn Fn(HtmlParseLimits) -> Result<(), Box<Stopped>>,
) -> (Vec<String>, Vec<Site>) {
    let unrestricted = limits(100);
    let _scope = PreparedSlotStopScope::begin();
    if let Err(stop) = parse(unrestricted) {
        panic!(
            "the unstopped parse completes, but stopped with {:?}",
            stop.stop
        );
    }
    let slots = prepared_slots();
    assert!(slots > 0, "the document prepares payload slots");
    let sites: Vec<Site> = prepared_slot_sites().into_iter().map(site).collect();
    let mut retained = Vec::new();
    let mut stops = 0;
    for slot in 0..slots {
        stop_after_prepared_slot(Some(slot));
        let stop = parse(unrestricted);
        stop_after_prepared_slot(None);
        let Err(stop) = stop else { continue };
        stops += 1;
        note_retained(&mut retained, &format!("after slot {slot}"), &stop);
        let at_stop = stop.at_stop.expect("a stopped parse reports its usage");
        for resource in [
            HtmlParseResource::WorkCredits,
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::AuxiliaryEntries,
        ] {
            let used = at_stop.resource(resource).current();
            let builder = unrestricted.into_builder();
            let capped = match resource {
                HtmlParseResource::WorkCredits => builder.work_credits(used),
                HtmlParseResource::ManagedCapacityBytes => builder.managed_capacity_bytes(used),
                _ => builder
                    .auxiliary_entries(used)
                    .active_formatting_entries(used),
            }
            .finish()
            .unwrap();
            if let Err(stop) = parse(capped) {
                note_retained(
                    &mut retained,
                    &format!("after slot {slot} with {resource:?} limited to {used}"),
                    &stop,
                );
            }
        }
    }
    assert!(stops > 0, "some stop is reached");
    (retained, sites)
}

/// The source file and the function that asked for a payload slot. The
/// function, not the line, names the site, so an unrelated edit above it does
/// not move it.
#[derive(Debug, Eq, PartialEq)]
struct Site {
    file: String,
    function: String,
}
/// The name `line` declares, when it begins a function.
fn declared_function(line: &str) -> Option<String> {
    let mut rest = line.trim_start();
    if let Some(after) = rest.strip_prefix("pub") {
        rest = after;
        if rest.starts_with('(') {
            rest = rest.split_once(')')?.1;
        }
        rest = rest.trim_start();
    }
    let name = rest.strip_prefix("fn ")?;
    Some(
        name.chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect(),
    )
}
fn site(location: &std::panic::Location<'_>) -> Site {
    let file = location.file().replace('\\', "/");
    let relative = file
        .split_once("/src/")
        .map_or(file.as_str(), |(_, rest)| rest);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(relative);
    let source = std::fs::read_to_string(&path).expect("the file of a recorded site is readable");
    let function = source
        .lines()
        .take(location.line() as usize)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .find_map(declared_function)
        .expect("a recorded site is inside a function");
    Site {
        file: format!("src/{relative}"),
        function,
    }
}

/// Requires that stopping `parse` after each of its prepared payload slots
/// leaves nothing retained, and that the slots it prepares include one asked
/// for by each of `reaches`, as `(file, function)`: a document that stops
/// reaching the site it is meant to cover would otherwise pass vacuously.
#[track_caller]
fn assert_nothing_retained(
    parse: &dyn Fn(HtmlParseLimits) -> Result<(), Box<Stopped>>,
    reaches: &[(&str, &str)],
) {
    let (retained, sites) = retained_after_each_prepared_slot(parse);
    assert!(
        retained.is_empty(),
        "{} stops retained something, first: {:?}",
        retained.len(),
        &retained[..retained.len().min(8)]
    );
    for (file, function) in reaches {
        assert!(
            sites
                .iter()
                .any(|site| site.file == *file && site.function == *function),
            "no prepared slot was asked for by {function} in {file}; sites reached: {:?}",
            sites
        );
    }
}

fn document(source: &'static str) -> impl Fn(HtmlParseLimits) -> Result<(), Box<Stopped>> {
    move |limits| {
        crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8(source), limits)
            .parse()
            .map(drop)
            .map_err(|failure| stopped(&failure))
    }
}

fn fragment_context() -> HtmlFragmentContext {
    let mut builder = HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 100_000, 1000, 100, 1000, 16_777_216),
        HtmlHostFactsRevision::fresh(),
    );
    let document = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_document_template_facts(
            &document,
            HtmlDocumentClass::Html,
            HtmlTemplateDocumentRole::Ordinary,
            HtmlCaptureTemplateCacheInput::Absent,
        )
        .unwrap();
    let div = builder
        .add_element(
            &document,
            QualifiedName::new(Namespace::html(), None, "div"),
            OrderedAttributes::default(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder.establish_root(&div).unwrap();
    builder.establish_no_shadow_root(&div).unwrap();
    let capture: HtmlCapture = builder.finish().unwrap();
    HtmlFragmentContext::element(&capture, &capture.subject_id(&div).unwrap()).unwrap()
}

fn fragment(source: &'static str) -> impl Fn(HtmlParseLimits) -> Result<(), Box<Stopped>> {
    move |limits| {
        crate::HtmlFragmentParseRequest::with_options(
            HtmlSource::utf8(source),
            fragment_context(),
            limits,
            HtmlParserScriptingMode::Inert,
            true,
        )
        .unwrap()
        .parse()
        .map(drop)
        .map_err(|failure| stopped(&failure))
    }
}

#[test]
fn dialog_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document("<!doctype html><dialog open>d</dialog>"),
        &[("src/construction/dialog.rs", "initialize_dialog")],
    );
}

#[test]
fn style_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document("<!doctype html><style>a{}</style>"),
        &[("src/construction/style.rs", "initialize_style")],
    );
}

#[test]
fn hyperlink_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document("<!doctype html><a href=x>l</a><area href=z>"),
        &[("src/construction/associated.rs", "initial_associated_state")],
    );
}

#[test]
fn image_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document("<!doctype html><img src=y>"),
        &[("src/construction/associated.rs", "initial_associated_state")],
    );
}

#[test]
fn reconstructed_formatting_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document("<!doctype html><a href=x>1<p>2</a>3<b nonce=n>4<p>5</b>6"),
        &[
            ("src/construction/associated.rs", "initial_associated_state"),
            (
                "src/construction/storage_strings.rs",
                "register_group_string_owned",
            ),
        ],
    );
}

#[test]
fn form_control_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document(
            "<!doctype html><form id=f><input name=a form=f><textarea>t</textarea><output>o</output><select><option>x</option></select><button></button></form>",
        ),
        &[
            (
                "src/construction/storage_strings.rs",
                "register_group_string_owned",
            ),
            ("src/construction/storage_allocation.rs", "register_raw"),
        ],
    );
}

#[test]
fn template_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document("<!doctype html><template><p>t</p><template><b>n</b></template></template>"),
        &[
            (
                "src/construction/template.rs",
                "prepare_document_allocation",
            ),
            (
                "src/construction/template.rs",
                "appropriate_contents_document",
            ),
        ],
    );
}

#[test]
fn script_link_and_details_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document(
            "<!doctype html><script>1</script><link rel=stylesheet href=s><details open><summary>s</summary></details>",
        ),
        &[
            (
                "src/construction/session_script.rs",
                "allocate_disabled_parser_script",
            ),
            (
                "src/construction/session_creation.rs",
                "allocate_for_creation",
            ),
        ],
    );
}

#[test]
fn text_select_and_style_order_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document(
            "<!doctype html><textarea>a&amp;b</textarea><select><option>a</option><option selected>b</option></select><style>a{}</style><style>b{}</style>",
        ),
        &[
            ("src/construction/text_append.rs", "append_registered_text"),
            (
                "src/construction/pending_association.rs",
                "retain_style_order",
            ),
            ("src/construction/effects.rs", "retain_work_list"),
            (
                "src/construction/storage_strings.rs",
                "register_group_string_owned",
            ),
        ],
    );
}

#[test]
fn base_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document("<!doctype html><base target=T><p>X"),
        &[("src/construction/base.rs", "register_base")],
    );
}

#[test]
fn meta_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document(
            "<!doctype html><meta http-equiv=content-security-policy content=1><meta charset=utf-8>",
        ),
        &[("src/construction/meta.rs", "meta_missing")],
    );
}

#[test]
fn nonce_and_svg_script_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document("<!doctype html><p nonce=n>x</p><svg><script>1</script><g nonce=m></g></svg>"),
        &[
            ("src/construction/nonce.rs", "prepare_nonce"),
            ("src/construction/svg.rs", "retain_svg_slot"),
            (
                "src/construction/storage_strings.rs",
                "register_group_string_owned",
            ),
        ],
    );
}

#[test]
fn attribute_and_cause_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document("<!doctype html><p id=a class=b>x<p id=c></b><table><tr><td>x</table>"),
        &[
            ("src/construction/provenance.rs", "prepare_node_causes"),
            (
                "src/construction/storage_strings.rs",
                "register_group_string_owned",
            ),
        ],
    );
}

#[test]
fn fragment_template_and_shadow_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot()
{
    assert_nothing_retained(
        &fragment(
            "<template><p>t</p></template><div><template shadowrootmode=open><slot></slot></template></div>",
        ),
        &[
            ("src/construction/context.rs", "attach_declarative_shadow"),
            (
                "src/construction/template.rs",
                "appropriate_contents_document",
            ),
        ],
    );
}

#[test]
fn fragment_association_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &fragment("<style media=screen>x{}</style><a href=x>l</a><script>1</script>"),
        &[
            (
                "src/construction/pending_association.rs",
                "retain_style_order",
            ),
            ("src/construction/style.rs", "initialize_style"),
            (
                "src/construction/session_script.rs",
                "allocate_disabled_parser_script",
            ),
            ("src/construction/associated.rs", "initial_associated_state"),
        ],
    );
}

#[test]
fn fragment_reconstructed_formatting_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot()
 {
    assert_nothing_retained(
        &fragment("<a href=x>1<p>2</a>3<form><input></form>"),
        &[
            (
                "src/construction/storage_strings.rs",
                "register_group_string_owned",
            ),
            (
                "src/construction/session_creation.rs",
                "allocate_for_creation",
            ),
        ],
    );
}

fn record(revision: &HtmlBaseFactsRevision, url: &str) -> HtmlBaseUrlRecord {
    HtmlBaseUrlRecord::observed(
        revision.clone(),
        url.into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap()
}

#[test]
fn associated_input_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = record(&revision, "https://example.test/");
    let encoding = HtmlBaseEncoding::fresh();
    let facts = HtmlEncodingUrlParseFacts::new(
        base.clone(),
        encoding.clone(),
        revision.clone(),
        ["x"]
            .into_iter()
            .map(|url| {
                HtmlEncodingUrlParseObservation::new(
                    url.into(),
                    HtmlEncodingUrlParseOutcome::Parsed(record(
                        &revision,
                        &format!("https://example.test/{url}"),
                    )),
                )
            })
            .collect(),
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap();
    let environment = HtmlParseBaseFacts::new(
        HtmlBaseDocumentFacts::new(
            revision,
            Some(base),
            HtmlAboutBaseUrl::KnownNull,
            false,
            Some(encoding),
        )
        .unwrap(),
        vec![],
        HtmlBaseFactsLimits::new(20, 10_000, 100_000),
    )
    .unwrap();
    assert_nothing_retained(
        &|limits| {
            crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype html><a href=x></a><img src=x crossorigin=anonymous referrerpolicy=origin>",
            ),
            limits,
        )
        .with_base_facts(environment.clone())
        .unwrap()
        .with_encoding_url_facts(vec![facts.clone()])
        .parse()
        .map(drop)
        .map_err(|failure| stopped(&failure))
        },
        &[
            (
                "src/construction/associated.rs",
                "install_associated_inputs",
            ),
            ("src/construction/base.rs", "install_document_base_facts"),
        ],
    );
}

/// A processor that attests every policy unchanged, so each reached Meta
/// admits the rows of a processed policy without any native artifact.
struct Unchanged {
    authority: HtmlCspProcessorAuthority,
    profile: HtmlCspProcessorProfile,
}
fn allow(_: HtmlCspLayout) -> Result<(), Infallible> {
    Ok(())
}
impl HtmlMetaCspProcessor for Unchanged {
    type Error = ();
    fn profile(&self) -> &HtmlCspProcessorProfile {
        &self.profile
    }
    fn prepare<E>(
        &mut self,
        request: &HtmlMetaCspRequest,
        admit: &mut impl FnMut(HtmlCspLayout) -> Result<(), E>,
        _: &mut impl FnMut(HtmlCspStorage),
    ) -> Result<HtmlMetaCspOutcome<()>, HtmlCspError<E>> {
        self.profile
            .validate_selected_with(request.qualification().processor(), admit)?;
        self.authority
            .attest_unchanged_with(request, admit)
            .map(HtmlMetaCspOutcome::Completed)
    }
}

#[test]
fn processed_policy_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &|limits| {
            let authority = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
            let profile = authority.profile_with(1, &mut allow).unwrap();
            let mut processor = Unchanged { authority, profile };
            crate::HtmlDocumentParseRequest::offline(
            HtmlSource::utf8(
                "<!doctype html><meta http-equiv=content-security-policy content='='><style>a{}</style><meta http-equiv=content-security-policy content='='>",
            ),
            limits,
        )
        .parse_with_meta_csp(&mut processor)
        .map(drop)
        .map_err(|failure| stopped(failure.parse_failure()))
        },
        &[
            ("src/construction/csp.rs", "service_meta_csp"),
            ("src/construction/csp.rs", "ensure_csp_policy"),
            ("src/construction/csp.rs", "enable_meta_csp"),
        ],
    );
}

#[test]
fn a_slot_dropped_without_its_row_is_returned_by_the_release() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    // One slot of a plain row, one that also admits the entry of a list.
    drop(storage.prepare_payload_slot_owned(&mut c).unwrap());
    drop(
        storage
            .prepare_payload_slot_for(&mut c, Some(KindList::Base))
            .unwrap(),
    );
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn a_row_of_another_kind_than_its_slot_admitted_is_refused_and_still_released() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    let rows = storage.payloads.values.len();
    // A slot admitted for a listed row refuses a row of no list.
    let prepared = storage
        .prepare_payload_slot_for(&mut c, Some(KindList::Base))
        .unwrap();
    assert!(
        storage
            .payloads
            .push_row(prepared, RetainedPayload::Retired)
            .is_err()
    );
    // The refused row is kept for the release, but is not listed.
    assert_eq!(storage.payloads.values.len(), rows + 1);
    assert!(storage.payloads.kind_list(KindList::Base).is_empty());
    session.release(&mut c).unwrap();
    storage_zero(&c);
}

#[test]
fn an_armed_stop_does_not_outlive_a_sweep_that_unwinds() {
    let unwound = std::panic::catch_unwind(|| {
        let _scope = PreparedSlotStopScope::begin();
        stop_after_prepared_slot(Some(0));
        panic!("a sweep fails with its stop armed");
    });
    assert!(unwound.is_err());
    // A stop left armed would cancel this parse at its first prepared slot.
    assert!(document("<!doctype html><p>x")(limits(100)).is_ok());
}

#[test]
fn processing_instruction_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document("<!doctype html><?target data?><p>x"),
        &[("src/construction/pi.rs", "prepare_pi_map")],
    );
}

#[test]
fn merged_text_rows_leave_nothing_retained_when_the_parse_stops_after_their_slot() {
    assert_nothing_retained(
        &document(
            "<!doctype html><textarea>a&amp;b&lt;c</textarea><style>x&amp;y</style><p>a&amp;b</p>",
        ),
        &[("src/construction/text_append.rs", "append_registered_text")],
    );
}

#[test]
fn a_refused_row_that_owns_a_string_is_released_with_the_registry() {
    let request = request("<body><div id=a>");
    let (mut c, mut session, _proof, _nodes) = graph(&request);
    let storage = session.active_mut().unwrap();
    let (value, allocation) = literal_string(&mut c, b"owned").unwrap().into_parts();
    // A slot admitted for a listed row refuses a row of no list.
    let prepared = storage
        .prepare_payload_slot_for(&mut c, Some(KindList::Base))
        .unwrap();
    let row = RetainedPayload::String {
        value,
        allocation,
        origins: None,
        location: FieldRef::Detached,
    };
    assert!(storage.payloads.push_row(prepared, row).is_err());
    session.release(&mut c).unwrap();
    storage_zero(&c);
}
