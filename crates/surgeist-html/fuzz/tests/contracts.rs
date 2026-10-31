#![forbid(unsafe_code)]
use surgeist_fuzz::{engine, *};
use surgeist_html_fuzz::*;
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseFailureReason, HtmlParseResource, HtmlParseStop, HtmlSource,
};

fn targets() -> [UnicodeTarget; 2] {
    [UnicodeTarget::document(), UnicodeTarget::fragment()]
}
#[test]
fn strict_unicode_codec_roundtrips_surrogates_and_rejects_noncanonical_inputs_before_use() {
    for target in targets() {
        let kind = if target.descriptor().id().as_str().ends_with("fragment") {
            ParseKind::Fragment
        } else {
            ParseKind::Document
        };
        let c = UnicodeCase::new(
            kind,
            UnicodeSource::Utf16(vec![0xfeff, 0xd800, 0xdc00, 0, 13, 10]),
            Lane::General,
        )
        .unwrap();
        let encoded = target.encode(&c).unwrap();
        assert_eq!(
            target.encode(&target.decode(&encoded).unwrap()).unwrap(),
            encoded
        );
        assert_eq!(
            target.decode(&encoded).unwrap().source(),
            &UnicodeSource::Utf16(vec![0xfeff, 0xd800, 0xdc00, 0, 13, 10])
        );
        for n in 0..encoded.len() {
            assert!(target.decode(&encoded[..n]).is_err(), "truncated at {n}");
        }
        for offset in [0, 4, 5, 6, 7] {
            let mut b = encoded.clone();
            b[offset] = 255;
            assert!(target.decode(&b).is_err(), "tag {offset}");
        }
        let mut b = encoded.clone();
        b.extend_from_slice(&[0]);
        assert!(target.decode(&b).is_err());
        let mut b = encoded.clone();
        b[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(target.decode(&b).is_err());
        let mut b = encoded.clone();
        b[160..164].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(target.decode(&b).is_err());
        assert!(
            UnicodeCase::new(kind, UnicodeSource::Utf16(vec![0; 4097]), Lane::General).is_err()
        );
        assert!(
            UnicodeCase::new(kind, UnicodeSource::Utf8("🙂".repeat(4097)), Lane::General).is_err()
        );
        let mut b = encoded.clone();
        b[5] = 0;
        b[160..164].copy_from_slice(&12u32.to_le_bytes());
        b[164] = 0xff;
        assert!(target.decode(&b).is_err());
    }
}
#[test]
fn literal_recovery_native_unicode_and_qualified_provenance_are_real_successes() {
    let mut ordinary_successes = 0;
    for target in targets() {
        for id in 0..9 {
            let c = target.literal(id).unwrap();
            let result = engine::evaluate_case(&target, &c, &mut |_| Ok(())).unwrap();
            assert_eq!(
                result.outcome.kind(),
                OutcomeKind::Evaluated,
                "{id}: {:?}",
                result.outcome
            );
            assert!(
                result
                    .outcome
                    .checked_properties()
                    .iter()
                    .any(|p| p.as_str() == "surgeist-html::literal-semantics")
            );
            let replay = engine::replay(&target, &result.canonical, &mut |bytes| {
                assert_eq!(bytes, result.canonical);
                Ok(())
            })
            .unwrap();
            assert_eq!(result.outcome, replay.outcome);
            ordinary_successes += 1;
        }
    }
    assert_eq!(ordinary_successes, 18);
}
#[test]
fn actual_cancel_source_and_work_refusals_have_retained_request_and_cleanup_properties() {
    for target in targets() {
        for lane in [Lane::PreCancelled, Lane::SourceDenied, Lane::WorkDenied] {
            let kind = target.literal(0).unwrap().kind();
            let c = UnicodeCase::new(kind, UnicodeSource::Utf16(vec![0xd800, 60, 112, 62]), lane)
                .unwrap();
            let result = target.evaluate(&c).unwrap();
            assert_eq!(result.kind(), OutcomeKind::ExpectedRejection, "{result:?}");
            assert!(
                result
                    .checked_properties()
                    .iter()
                    .any(|p| p.as_str() == "surgeist-html::terminal-ownership")
            );
        }
    }
    let c = UnicodeCase::new(
        ParseKind::Fragment,
        UnicodeSource::Utf8("<p>x".into()),
        Lane::MissingScripting,
    )
    .unwrap();
    assert_eq!(
        UnicodeTarget::fragment().evaluate(&c).unwrap().kind(),
        OutcomeKind::ExpectedRejection
    );
    assert!(
        UnicodeCase::new(
            ParseKind::Document,
            UnicodeSource::Utf8("x".into()),
            Lane::MissingScripting
        )
        .is_err()
    );
}
#[test]
fn independent_literal_oracles_cannot_be_applied_to_different_input() {
    assert!(
        UnicodeCase::new(
            ParseKind::Document,
            UnicodeSource::Utf8("<p>wrong".into()),
            Lane::Literal(0)
        )
        .is_err()
    );
    let c = UnicodeTarget::document().literal(0).unwrap();
    assert!(UnicodeTarget::fragment().encode(&c).is_err());
    assert!(UnicodeTarget::fragment().evaluate(&c).is_err());
}
#[test]
fn generation_replay_and_reduction_keep_actual_units_limits_and_operations() {
    for target in targets() {
        let mut success = 0;
        for first in [0, 4, 8, 12, 16, 20, 24, 28, 32, 1, 2, 3] {
            let e = [first, 0, 0xd8, 0, 0, 13, 0, 10, 0];
            let result = engine::evaluate_generated(&target, &e, &mut |_| Ok(())).unwrap();
            let engine::GeneratedEvaluation::Evaluated(r) = result else {
                panic!("bounded generation discarded")
            };
            assert_ne!(
                r.outcome.kind(),
                OutcomeKind::Failure,
                "{first}: {:?}",
                r.outcome
            );
            if r.outcome.kind() == OutcomeKind::Evaluated {
                success += 1;
            }
            assert_eq!(
                engine::replay(&target, &r.canonical, &mut |_| Ok(()))
                    .unwrap()
                    .outcome,
                r.outcome
            );
        }
        assert!(success >= 8);
        let kind = target.literal(0).unwrap().kind();
        let c = UnicodeCase::new(
            kind,
            UnicodeSource::Utf8("🙂é<p>X".into()),
            Lane::PreCancelled,
        )
        .unwrap();
        let mut calls = 0;
        target
            .reduction_candidates(&c, &mut |candidate| {
                assert_eq!(candidate.lane(), Lane::PreCancelled);
                assert_eq!(candidate.limits().unwrap(), c.limits().unwrap());
                calls += 1;
                false
            })
            .unwrap();
        assert_eq!(calls, 1);
        let mut candidates = 0;
        target
            .reduction_candidates(&c, &mut |candidate| {
                assert!(target.encode(&candidate).is_ok());
                assert_eq!(
                    target.evaluate(&candidate).unwrap().kind(),
                    OutcomeKind::ExpectedRejection
                );
                candidates += 1;
                true
            })
            .unwrap();
        assert_eq!(candidates, 3);
        assert!(matches!(
            target.generate(&vec![1; 9000]).unwrap(),
            Generation::Rejected(_)
        ));
    }
}
#[cfg(feature = "property")]
#[test]
fn finite_property_engine_evaluates_exact_literal_case_and_records_nonvacuous_success() {
    use proptest::prelude::*;
    for target in targets() {
        let mut captured = 0;
        let r = engine::run_property(
            &target,
            &Just(vec![0]),
            engine::PropertyConfig {
                cases: 4,
                max_rejections: 0,
                max_shrink_iterations: 8,
                seed: 1139,
            },
            &mut |_| {
                captured += 1;
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(r.counts.evaluated(), 4);
        assert!(r.failure.is_none());
        assert_eq!(captured, 4);
    }
}
#[cfg(feature = "coverage")]
#[test]
fn coverage_driver_and_replay_evaluate_the_same_canonical_case() {
    for target in targets() {
        let mut driver = engine::CoverageDriver::new(&target, None).unwrap();
        let mut captured = Vec::new();
        let actual = driver
            .test_one(&[0], &mut |b| {
                captured = b.to_vec();
                Ok(())
            })
            .unwrap();
        let engine::GeneratedEvaluation::Evaluated(actual) = actual else {
            panic!("literal rejected")
        };
        let replay = engine::replay(&target, &captured, &mut |_| Ok(())).unwrap();
        assert_eq!(actual, replay);
    }
}

fn exchange(
    target: &UnicodeTarget,
    request: surgeist_fuzz::worker::WorkerRequest,
) -> Vec<surgeist_fuzz::worker::WorkerResponse> {
    use surgeist_fuzz::worker::*;
    let mut input = Vec::new();
    write_frame(&mut input, &request, 65536).unwrap();
    let mut out = Vec::new();
    serve_io(
        target,
        &mut std::io::Cursor::new(input),
        &mut out,
        WorkerLimits::new(65536, 65536, 262144, 4).unwrap(),
    )
    .unwrap();
    let mut input = std::io::Cursor::new(out);
    let mut responses = Vec::new();
    while let Some(r) = read_frame(&mut input, 65536).unwrap() {
        responses.push(r);
    }
    responses
}
#[test]
fn framework_worker_captures_before_evaluating_and_reduces_checked_canonical_cases() {
    use surgeist_fuzz::worker::*;
    for t in targets() {
        let bytes = t.encode(&t.literal(0).unwrap()).unwrap();
        let r = exchange(
            &t,
            WorkerRequest::Evaluate {
                canonical: bytes.clone(),
            },
        );
        assert!(
            matches!(&r[..],[WorkerResponse::Captured{canonical},WorkerResponse::Evaluated{outcome,..}] if *canonical==bytes && outcome.kind()==OutcomeKind::Evaluated)
        );
        assert!(
            matches!(&exchange(&t,WorkerRequest::Descriptor)[..],[WorkerResponse::Descriptor{descriptor}] if descriptor==t.descriptor())
        );
        let c = UnicodeCase::new(
            t.literal(0).unwrap().kind(),
            UnicodeSource::Utf16(vec![0xd800, 60, 112, 62]),
            Lane::PreCancelled,
        )
        .unwrap();
        let r = exchange(
            &t,
            WorkerRequest::Reduce {
                canonical: t.encode(&c).unwrap(),
                max_candidates: 2,
            },
        );
        assert!(matches!(
            r.last(),
            Some(WorkerResponse::Reduced { emitted: 2 })
        ));
        for r in &r[..r.len() - 1] {
            let WorkerResponse::Candidate { canonical } = r else {
                panic!("reduction protocol")
            };
            assert_eq!(
                engine::replay(&t, canonical, &mut |_| Ok(()))
                    .unwrap()
                    .outcome
                    .kind(),
                OutcomeKind::ExpectedRejection
            );
        }
        assert!(matches!(
            t.generate(&vec![2; 6000]).unwrap(),
            Generation::Rejected(_)
        ));
    }
}
/// Handwritten literal inputs (UTF-8 text, or UTF-16 units for the surrogate seed),
/// stated from the pinned HTML contract and not read from the codec or seed module.
const SEED_TEXT: [&str; 9] = [
    "<p ID=x id=y>A&amp;B\r\nC</p>",
    "",
    "<p>A\0B</p>",
    "<svg><a xlink:href='x' viewBox='0 0 1 1'></a></svg>",
    "<template><p>T</p></template><p>Z</p>",
    "<?target data?><p>P</p>",
    "<p>A<p>B",
    "<table>X<tr><td>T</td></tr></table>",
    "<?target data><p>P</p>",
];
const SEED_SURROGATE_UNITS: [u16; 13] =
    [0xfeff, 60, 112, 62, 65, 0xd800, 13, 10, 66, 60, 47, 112, 62];
/// The README "Cases and codec" header: 4 + 1 + 1 + 2 + 19 * 8 + 4 bytes.
const README_HEADER_LEN: usize = 164;
/// The documented generous product envelope every literal case carries.
const README_ENVELOPE: [u64; 19] = [
    100_000,
    100_000,
    10_000,
    1_000_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100_000,
    100,
    100_000,
    100_000_000,
    100_000_000,
    100_000_000,
    100_000_000,
];

#[test]
fn seed_files_decode_to_the_documented_layout_and_independent_literal_success() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for (name, target_byte, t) in [
        ("document", 0u8, UnicodeTarget::document()),
        ("fragment", 1u8, UnicodeTarget::fragment()),
    ] {
        let mut files = std::fs::read_dir(root.join("seeds").join(name))
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        files.sort();
        let expected_files = (0..9).map(|id| format!("{id}.case")).collect::<Vec<_>>();
        assert_eq!(files, expected_files, "{name} seed inventory");
        for id in 0..9u8 {
            let b = std::fs::read(root.join(format!("seeds/{name}/{id}.case"))).unwrap();
            assert!(b.len() >= README_HEADER_LEN, "{name}/{id} header");
            assert_eq!(&b[0..4], b"HUF1", "{name}/{id} magic");
            assert_eq!(b[4], target_byte, "{name}/{id} target");
            // Source tag, lane 4 (independent literal) and the literal ID.
            let utf16 = id == 1;
            assert_eq!(b[5], u8::from(utf16), "{name}/{id} source tag");
            assert_eq!(&b[6..8], &[4, id], "{name}/{id} lane and literal id");
            let caps: Vec<u64> = (0..19)
                .map(|i| u64::from_le_bytes(b[8 + i * 8..16 + i * 8].try_into().unwrap()))
                .collect();
            assert_eq!(caps, README_ENVELOPE, "{name}/{id} parser caps");
            let count = u32::from_le_bytes(b[160..164].try_into().unwrap()) as usize;
            let payload = &b[README_HEADER_LEN..];
            if utf16 {
                let units: Vec<u16> = payload
                    .chunks_exact(2)
                    .map(|p| u16::from_le_bytes([p[0], p[1]]))
                    .collect();
                assert_eq!(payload.len() % 2, 0, "{name}/{id} whole units");
                assert_eq!(count, units.len(), "{name}/{id} unit count");
                assert_eq!(units, SEED_SURROGATE_UNITS, "{name}/{id} payload units");
            } else {
                assert_eq!(count, payload.len(), "{name}/{id} byte count");
                assert_eq!(
                    payload,
                    SEED_TEXT[usize::from(id)].as_bytes(),
                    "{name}/{id} payload"
                );
            }
            assert_eq!(
                engine::replay(&t, &b, &mut |_| Ok(()))
                    .unwrap()
                    .outcome
                    .kind(),
                OutcomeKind::Evaluated,
                "{name}/{id}"
            );
        }
    }
}

#[test]
fn processing_instruction_literals_use_each_documented_ending() {
    // HTML 877d 13.1.7: a processing instruction ends with `?>` or `>`.
    for target in targets() {
        assert_eq!(
            target.literal(5).unwrap().source(),
            &UnicodeSource::Utf8("<?target data?><p>P</p>".into())
        );
        assert_eq!(
            target.literal(8).unwrap().source(),
            &UnicodeSource::Utf8("<?target data><p>P</p>".into())
        );
        assert!(target.literal(9).is_err());
    }
}

#[test]
fn descriptor_source_references_name_existing_repository_files() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let descriptor: toml::Table = toml::from_str(
        &std::fs::read_to_string(env!("CARGO_MANIFEST_DIR").to_owned() + "/fuzz.toml").unwrap(),
    )
    .unwrap();
    let descriptor_targets = descriptor["targets"].as_array().unwrap();
    assert_eq!(descriptor_targets.len(), 2);
    for target in descriptor_targets {
        let id = target["id"].as_str().unwrap();
        let references = target["source_references"].as_array().unwrap();
        assert!(!references.is_empty(), "{id}");
        for reference in references {
            let reference = reference.as_str().unwrap();
            let path = reference.split('#').next().unwrap();
            assert!(
                root.join(path).is_file(),
                "{id} source reference {reference:?} is not a file under the repository root"
            );
        }
    }
}

#[test]
fn owner_package_registers_real_workers_and_canonical_seed_directories() {
    use surgeist_fuzz::registry::TargetRegistry;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let metadata = std::process::Command::new(env!("CARGO"))
        .current_dir(&root)
        .args([
            "metadata",
            "--offline",
            "--locked",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .output()
        .unwrap();
    assert!(
        metadata.status.success(),
        "{}",
        String::from_utf8_lossy(&metadata.stderr)
    );
    let registry = TargetRegistry::from_metadata_json(&root, &metadata.stdout).unwrap();
    for (id, binary) in [
        ("surgeist-html::unicode-document", "html-document-worker"),
        ("surgeist-html::unicode-fragment", "html-fragment-worker"),
    ] {
        let target = registry.get(&TargetId::new(id).unwrap()).unwrap();
        assert_eq!(target.package(), "surgeist-html-fuzz");
        assert_eq!(target.descriptor().binary(), binary);
        assert_eq!(target.fixtures().len(), 2);
        assert!(target.fixtures().iter().all(|p| p.is_dir()));
    }
}
#[cfg(not(feature = "coverage"))]
#[test]
fn actual_ordinary_worker_binaries_read_canonical_frames_and_publish_capture_then_success() {
    use std::io::Write;
    use surgeist_fuzz::worker::*;
    for (binary, t) in [
        (
            env!("CARGO_BIN_EXE_html-document-worker"),
            UnicodeTarget::document(),
        ),
        (
            env!("CARGO_BIN_EXE_html-fragment-worker"),
            UnicodeTarget::fragment(),
        ),
    ] {
        let canonical = t.encode(&t.literal(0).unwrap()).unwrap();
        let mut bytes = Vec::new();
        write_frame(
            &mut bytes,
            &WorkerRequest::Evaluate {
                canonical: canonical.clone(),
            },
            65536,
        )
        .unwrap();
        let mut child = std::process::Command::new(binary)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&bytes).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut reader = std::io::Cursor::new(output.stdout);
        assert!(
            matches!(read_frame::<_,WorkerResponse>(&mut reader,65536).unwrap(),Some(WorkerResponse::Captured{canonical:b}) if b==canonical)
        );
        assert!(
            matches!(read_frame::<_,WorkerResponse>(&mut reader,65536).unwrap(),Some(WorkerResponse::Evaluated{outcome,..}) if outcome.kind()==OutcomeKind::Evaluated)
        );
        assert!(
            read_frame::<_, WorkerResponse>(&mut reader, 65536)
                .unwrap()
                .is_none()
        );
    }
}

#[test]
fn finite_nonzero_work_cut_checks_partial_execution_cleanup_and_retained_evidence() {
    let caps = [
        100000, 100000, 10000, 1000000, 100000, 100000, 100000, 100000, 100000, 100000, 100000,
        100000, 100000, 100, 100000, 100000000, 100000000, 100000000, 199923,
    ];
    for t in targets() {
        let c = UnicodeCase::new(
            t.literal(0).unwrap().kind(),
            UnicodeSource::Utf8(
                "<form id=f><input type=hidden value=x><textarea>A&amp;B</textarea></form>".into(),
            ),
            Lane::General,
        )
        .unwrap()
        .with_caps(caps)
        .unwrap();
        let outcome = t.evaluate(&c).unwrap();
        assert_eq!(
            outcome.kind(),
            OutcomeKind::ExpectedRejection,
            "{outcome:?}"
        );
        assert!(
            outcome
                .checked_properties()
                .iter()
                .any(|p| p.as_str() == "surgeist-html::terminal-ownership")
        );
    }
}

#[test]
fn public_partial_work_failure_has_admitted_source_first_stop_and_actual_cleanup_snapshot() {
    let c = UnicodeCase::new(
        ParseKind::Document,
        UnicodeSource::Utf8(
            "<form id=f><input type=hidden value=x><textarea>A&amp;B</textarea></form>".into(),
        ),
        Lane::General,
    )
    .unwrap();
    let limits = c
        .limits()
        .unwrap()
        .into_builder()
        .work_credits(199923)
        .finish()
        .unwrap();
    let UnicodeSource::Utf8(s) = c.source() else {
        panic!("literal source")
    };
    let request = HtmlDocumentParseRequest::offline(HtmlSource::utf8(s.clone()), limits);
    let cancel = request.cancellation().clone();
    let f = request.parse().unwrap_err();
    assert_eq!(f.source_admitted(), Some(true));
    let HtmlParseFailureReason::Stop(
        stop @ HtmlParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            ..
        },
    ) = f.reason()
    else {
        panic!("{f:?}")
    };
    assert_eq!(f.first_stop(), Some(stop));
    let retained = f.retained_usage().unwrap().clone();
    let at = f.at_stop().unwrap().clone();
    let after = f.after_cleanup().unwrap().clone();
    assert!(at.resource(HtmlParseResource::WorkCredits).current() > 0);
    for r in [
        HtmlParseResource::BufferedUtf16Units,
        HtmlParseResource::TokenAttributes,
        HtmlParseResource::ManagedEdges,
        HtmlParseResource::ManagedAttributes,
        HtmlParseResource::ManagedUtf16Units,
        HtmlParseResource::OpenElements,
    ] {
        assert_eq!(after.resource(r).current(), 0);
    }
    assert_eq!(f.cleanup_error(), None);
    cancel.cancel();
    assert_eq!(f.first_stop(), Some(stop));
    assert_eq!(f.retained_usage(), Some(&retained));
    assert_eq!(f.at_stop(), Some(&at));
    assert_eq!(f.after_cleanup(), Some(&after));
}

#[test]
fn shared_failure_reducer_refuses_to_minimize_a_real_successful_case() {
    use surgeist_fuzz::reduction::*;
    for t in targets() {
        let bytes = t.encode(&t.literal(0).unwrap()).unwrap();
        let key = FailureKey::new(
            t.descriptor().properties()[5].clone(),
            FailureClass::PropertyViolation,
            Some("literal-semantics".into()),
        )
        .unwrap();
        let report = reduce_direct(
            &t,
            &bytes,
            &key,
            ReductionLimits::new(8, std::time::Duration::from_secs(5), 65536, 2).unwrap(),
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(report.status(), ReductionStatus::OriginalNotReproduced);
        assert_eq!(report.original_input(), bytes);
        assert!(!report.original_confirmed());
    }
}
