//! Evaluation of a markup recipe through the public parse front doors.
use super::model::{CONTEXTS, Ns};
use super::{Body, Case, MarkupTarget, Mode, compare, tree};
use crate::captured::{Verdict, check};
use crate::observe::{
    Expect, Props, Spec, context_for, verify_failure, verify_immutable, verify_report,
};
use surgeist_fuzz::*;
use surgeist_html::bounded::construction::HtmlConstructionResult;
use surgeist_html::{
    parsing::HtmlParserScriptingMode,
    values::{DocumentString, Namespace},
};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlFragmentParseRequest, HtmlParseCancellation, HtmlParseFailure,
    HtmlParseReport, HtmlSource,
};

/// Property slots: the five shared parse slots, the tree oracle, then the
/// capture mirror, fact qualification and atomic refusal of the capture target.
pub(crate) const TREE: usize = 5;
const CAPTURE: usize = 6;

pub(crate) fn source_of(case: &Case) -> HtmlSource {
    match case.source() {
        crate::UnicodeSource::Utf8(s) => HtmlSource::utf8(s),
        crate::UnicodeSource::Utf16(u) => HtmlSource::utf16(DocumentString::from_utf16(u)),
    }
}
pub(crate) fn context_ns(ns: Ns) -> Namespace {
    match ns {
        Ns::Html => Namespace::html(),
        Ns::Svg => Namespace::svg(),
        Ns::Math => Namespace::from_uri(ns.uri()),
    }
}

/// A parse request of either kind, so one path covers pre-cancellation,
/// parsing and the late-cancel comparison.
enum Request {
    Document(HtmlDocumentParseRequest),
    Fragment(HtmlFragmentParseRequest),
}
impl Request {
    fn cancellation(&self) -> HtmlParseCancellation {
        match self {
            Self::Document(r) => r.cancellation().clone(),
            Self::Fragment(r) => r.cancellation().clone(),
        }
    }
    #[expect(
        clippy::result_large_err,
        reason = "the parser returns its failure by value to retain its request without allocating"
    )]
    fn parse(self) -> std::result::Result<HtmlParseReport, HtmlParseFailure> {
        match self {
            Self::Document(r) => r.parse(),
            Self::Fragment(r) => r.parse(),
        }
    }
}

pub(crate) fn evaluate_parse(target: &MarkupTarget, case: &Case) -> Result<CaseOutcome> {
    let props = Props(&target.descriptor);
    let original = source_of(case);
    let text = case.source();
    let limits = crate::codec::limits(case.caps)?;
    let fragment = matches!(case.mode, Mode::Fragment { .. });
    let requested = if fragment {
        HtmlParserScriptingMode::Inert
    } else {
        HtmlParserScriptingMode::Disabled
    };
    let mut capture = None;
    let request = match case.mode {
        Mode::Document { .. } => Request::Document(
            HtmlDocumentParseRequest::offline(original.clone(), limits)
                .with_detail(surgeist_html_parser::DetailSet::ALL)
                .map_err(crate::observe::detail_error)?,
        ),
        Mode::Fragment { context, shadow } => {
            let info = &CONTEXTS[usize::from(context)];
            let cx = context_for(context_ns(info.ns), info.local, false)?;
            capture = Some(cx.capture().identity().clone());
            Request::Fragment(
                HtmlFragmentParseRequest::with_options(
                    original.clone(),
                    cx,
                    limits,
                    requested,
                    shadow,
                )
                .map_err(|e| {
                    FuzzError::with_source(
                        FuzzErrorKind::Harness,
                        "build fragment request",
                        "qualified capture",
                        e,
                    )
                })?
                .with_detail(surgeist_html_parser::DetailSet::ALL)
                .map_err(crate::observe::detail_error)?,
            )
        }
    };
    let cancel = request.cancellation();
    if case.cancelled {
        cancel.cancel();
    }
    let parsed = request.parse();
    let spec = Spec {
        fragment,
        text: &text,
        original: &original,
        limits: &limits,
        caps: case.caps,
        requested,
        capture: capture.as_ref(),
        shadow_allowed: case.shadow_permitted(),
    };
    match parsed {
        Ok(report) => {
            if case.cancelled {
                return props.failure(
                    3,
                    "missing-terminal-refusal",
                    "explicit stopped lane completed",
                );
            }
            let seen = match verify_report(&props, &spec, &report)? {
                Ok(v) => v,
                Err(outcome) => return Ok(outcome),
            };
            if let Err(key) = compare::general(&seen.nodes) {
                return props.failure(1, key, "parsed tree invariant");
            }
            let exact = match (&case.body, case.mode) {
                (Body::Exact(items), Mode::Document { doctype, .. }) => {
                    Some(tree::expect_document(doctype, items))
                }
                (Body::Exact(items), Mode::Fragment { .. }) => case
                    .root_class()
                    .map(|class| tree::expect_fragment(items, class, case.shadow_permitted())),
                (Body::Soup(_), _) => None,
            };
            if let Some(expected) = &exact
                && let Err(key) = compare::tree(&seen.nodes, expected)
            {
                return props.failure(TREE, key, "parsed tree differs from the recipe");
            }
            let mut bounded = false;
            if let Some(plan) = &case.plan {
                let result = match (report.document(), report.fragment()) {
                    (Some(d), _) => HtmlConstructionResult::Document(d.clone()),
                    (_, Some(f)) => HtmlConstructionResult::Fragment(f.clone()),
                    _ => {
                        return props.failure(1, "missing-native-root", "report without a root");
                    }
                };
                match check(&result, plan)? {
                    Verdict::Violation {
                        property,
                        key,
                        detail,
                    } => return props.failure(CAPTURE + property, key, detail),
                    Verdict::ResourceStop => bounded = true,
                    Verdict::Verified => {}
                }
            }
            if let Some(outcome) = verify_immutable(&props, &report, &seen, &cancel)? {
                return Ok(outcome);
            }
            let mut checked = vec![0, 1, 2, 4];
            if exact.is_some() {
                checked.push(TREE);
            }
            if case.plan.is_some() {
                checked.extend([CAPTURE, CAPTURE + 1, CAPTURE + 2]);
            }
            if bounded {
                // An adapter work allowance stopped the import; the refusal was
                // atomic and a fault-free retry completed.
                return CaseOutcome::expected_rejection(
                    props.ids(&checked),
                    "capture adapter work allowance",
                );
            }
            CaseOutcome::evaluated(props.ids(&checked))
        }
        Err(f) => {
            let expect = if case.cancelled {
                Expect::PreCancelled
            } else if matches!(case.body, Body::Exact(_)) {
                // A balanced recipe is expected to parse; only a stop at the
                // configured cap (or allocation or representation) is a
                // product outcome, and unsupported states are failures.
                Expect::Balanced
            } else {
                Expect::General
            };
            verify_failure(&props, &spec, &f, &cancel, expect)
        }
    }
}
