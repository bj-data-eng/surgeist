#![forbid(unsafe_code)]

use surgeist_fuzz::worker::{ResponseDecoder, WorkerResponse, write_frame};
use surgeist_fuzz::{FuzzError, FuzzErrorKind};

fn frame(json: &str) -> Vec<u8> {
    let mut bytes = (json.len() as u32).to_be_bytes().to_vec();
    bytes.extend_from_slice(json.as_bytes());
    bytes
}

#[test]
fn every_header_and_payload_split_preserves_order_and_coalesced_frames() {
    let first = WorkerResponse::Captured {
        canonical: vec![1, 2, 3],
    };
    let second = WorkerResponse::Generated { canonical: vec![9] };
    let mut bytes = Vec::new();
    write_frame(&mut bytes, &first, 1024).unwrap();
    write_frame(&mut bytes, &second, 1024).unwrap();
    for split in 0..=bytes.len() {
        let mut decoder = ResponseDecoder::new(1024).unwrap();
        let mut responses = Vec::new();
        let mut emit = |response| {
            responses.push(response);
            Ok(())
        };
        decoder.push(&bytes[..split], &mut emit).unwrap();
        decoder.push(&bytes[split..], &mut emit).unwrap();
        decoder.finish().unwrap();
        assert_eq!(responses, [first.clone(), second.clone()], "split {split}");
    }
    let mut decoder = ResponseDecoder::new(1024).unwrap();
    let mut responses = Vec::new();
    for byte in bytes {
        decoder
            .push(&[byte], &mut |response| {
                responses.push(response);
                Ok(())
            })
            .unwrap();
    }
    decoder.finish().unwrap();
    assert_eq!(responses, [first, second]);
}

#[test]
fn exact_frame_ceiling_is_accepted_and_larger_header_rejected_without_payload() {
    let bytes = frame(r#"{"status":"generated","canonical":[3]}"#);
    let maximum = (bytes.len() - 4) as u32;
    let mut decoder = ResponseDecoder::new(maximum).unwrap();
    let mut responses = Vec::new();
    decoder
        .push(&bytes, &mut |response| {
            responses.push(response);
            Ok(())
        })
        .unwrap();
    decoder.finish().unwrap();
    assert_eq!(
        responses,
        [WorkerResponse::Generated { canonical: vec![3] }]
    );
    for header in [0_u32, maximum + 1, u32::MAX] {
        let mut decoder = ResponseDecoder::new(maximum).unwrap();
        let error = decoder
            .push(&header.to_be_bytes(), &mut |_| {
                panic!("invalid frame emitted")
            })
            .unwrap_err();
        assert_eq!(error.kind(), FuzzErrorKind::Resource);
        assert!(decoder.push(&[], &mut |_| Ok(())).is_err());
    }
    assert!(ResponseDecoder::new(0).is_err());
}

#[test]
fn invalid_strict_json_keeps_preceding_capture_and_poisons_decoder() {
    let valid = frame(r#"{"status":"captured","canonical":[7]}"#);
    for json in [
        r#"{"status":"generated","canonical":[3],"extra":true}"#,
        r#"{"status":"generated","canonical":[3],"canonical":[4]}"#,
        r#"{"status":"generated","status":"captured","canonical":[3]}"#,
        r#"{"status":"generated"}"#,
        r#"{"status":"generated","canonical":[256]}"#,
        r#"{"status":"generated","canonical":[3]} null"#,
        r#"{"status":"unknown","canonical":[3]}"#,
        "{",
    ] {
        let mut decoder = ResponseDecoder::new(1024).unwrap();
        let mut bytes = valid.clone();
        bytes.extend(frame(json));
        let mut responses = Vec::new();
        let error = decoder
            .push(&bytes, &mut |response| {
                responses.push(response);
                Ok(())
            })
            .unwrap_err();
        assert_eq!(error.kind(), FuzzErrorKind::Codec, "{json}");
        assert_eq!(responses, [WorkerResponse::Captured { canonical: vec![7] }]);
        assert!(decoder.finish().is_err());
        assert!(decoder.push(&valid, &mut |_| Ok(())).is_err());
    }
}

#[test]
fn truncated_final_frames_keep_completed_capture() {
    let valid = frame(r#"{"status":"captured","canonical":[7]}"#);
    let trailing = frame(r#"{"status":"generated","canonical":[3]}"#);
    for retained in 1..trailing.len() {
        let mut decoder = ResponseDecoder::new(1024).unwrap();
        let mut bytes = valid.clone();
        bytes.extend_from_slice(&trailing[..retained]);
        let mut responses = Vec::new();
        decoder
            .push(&bytes, &mut |response| {
                responses.push(response);
                Ok(())
            })
            .unwrap();
        assert!(decoder.finish().is_err(), "retained {retained}");
        assert_eq!(responses, [WorkerResponse::Captured { canonical: vec![7] }]);
    }
    ResponseDecoder::new(1024).unwrap().finish().unwrap();
}

#[test]
fn consumer_failure_stops_coalesced_delivery_and_preserves_original_error() {
    let mut bytes = frame(r#"{"status":"captured","canonical":[7]}"#);
    bytes.extend(frame(r#"{"status":"generated","canonical":[3]}"#));
    let mut decoder = ResponseDecoder::new(1024).unwrap();
    let mut current = None;
    let mut calls = 0;
    let error = decoder
        .push(&bytes, &mut |response| {
            calls += 1;
            current = Some(response);
            Err(FuzzError::new(
                FuzzErrorKind::Artifact,
                "retain capture",
                "store full",
            ))
        })
        .unwrap_err();
    assert_eq!(calls, 1);
    assert_eq!(
        current,
        Some(WorkerResponse::Captured { canonical: vec![7] })
    );
    assert_eq!(error.kind(), FuzzErrorKind::Artifact);
    assert_eq!(error.operation(), "retain capture");
    assert!(decoder.push(&[], &mut |_| Ok(())).is_err());
}
