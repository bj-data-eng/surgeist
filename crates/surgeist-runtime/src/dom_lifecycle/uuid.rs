//! HTML's two independently generated session-history UUID fields.
//!
//! A single fallible 32-byte native fill supplies disjoint 16-byte generations.
//! The fixed UTF-16 arrays are the actual strings retained in the entry owner;
//! their 72 code units are included inline in that owner's admitted storage.
use super::DomHostedRuntimeError;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Eq, PartialEq)]
pub(super) struct HistoryUuids {
    pub key: [u16; 36],
    pub id: [u16; 36],
}
impl HistoryUuids {
    pub fn sample(
        retained: &mut Option<Self>,
        cancel: &AtomicBool,
    ) -> Result<(), DomHostedRuntimeError> {
        generate(retained, native_fill, cancel)
    }
}

fn generate(
    retained: &mut Option<HistoryUuids>,
    fill: impl FnOnce(&mut [u8; 32]) -> Result<(), DomHostedRuntimeError>,
    cancel: &AtomicBool,
) -> Result<(), DomHostedRuntimeError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(DomHostedRuntimeError::Cancelled);
    }
    if retained.is_some() {
        return Ok(());
    }
    let mut bytes = [0; 32];
    fill(&mut bytes)?;
    let mut key = [0; 16];
    let mut id = [0; 16];
    key.copy_from_slice(&bytes[..16]);
    id.copy_from_slice(&bytes[16..]);
    // Successful entropy generation establishes both fields, even if the
    // caller is cancelled immediately afterward. The original creation cursor
    // owns this envelope, so a retry must reuse these actual generations.
    *retained = Some(HistoryUuids {
        key: format(key),
        id: format(id),
    });
    if cancel.load(Ordering::Relaxed) {
        return Err(DomHostedRuntimeError::Cancelled);
    }
    Ok(())
}

#[cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]
fn native_fill(bytes: &mut [u8; 32]) -> Result<(), DomHostedRuntimeError> {
    getrandom::fill(bytes).map_err(|_| DomHostedRuntimeError::EntropyFailed)
}
#[cfg(not(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
)))]
fn native_fill(_: &mut [u8; 32]) -> Result<(), DomHostedRuntimeError> {
    Err(DomHostedRuntimeError::EntropyUnavailable)
}

fn format(mut bytes: [u8; 16]) -> [u16; 36] {
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let mut text = [0; 36];
    let mut output = 0;
    for (index, byte) in bytes.into_iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            text[output] = u16::from(b'-');
            output += 1;
        }
        for nibble in [byte >> 4, byte & 0xf] {
            text[output] = u16::from(if nibble < 10 {
                b'0' + nibble
            } else {
                b'a' + nibble - 10
            });
            output += 1;
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_webcrypto_uuid_bytes_have_exact_lowercase_order_version_and_variant() {
        for (bytes, expected) in [
            ([0; 16], "00000000-0000-4000-8000-000000000000"),
            ([255; 16], "ffffffff-ffff-4fff-bfff-ffffffffffff"),
            (
                [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
                "00010203-0405-4607-8809-0a0b0c0d0e0f",
            ),
        ] {
            assert_eq!(
                format(bytes).as_slice(),
                expected.encode_utf16().collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn one_native_fill_consumes_two_disjoint_generations_without_collision_policy() {
        let mut fills = 0;
        let mut retained = None;
        generate(
            &mut retained,
            |bytes| {
                fills += 1;
                for (index, byte) in bytes.iter_mut().enumerate() {
                    *byte = index as u8;
                }
                Ok(())
            },
            &AtomicBool::new(false),
        )
        .unwrap();
        let pair = retained.unwrap();
        assert_eq!(fills, 1);
        assert_eq!(
            String::from_utf16(&pair.key).unwrap(),
            "00010203-0405-4607-8809-0a0b0c0d0e0f"
        );
        assert_eq!(
            String::from_utf16(&pair.id).unwrap(),
            "10111213-1415-4617-9819-1a1b1c1d1e1f"
        );
        // Equal random generations remain valid. There is no collision retry.
        let mut retained = None;
        generate(&mut retained, |_| Ok(()), &AtomicBool::new(false)).unwrap();
        let same = retained.unwrap();
        assert_eq!(same.key, same.id);
    }

    #[test]
    fn partial_native_failure_and_cancel_do_not_publish_either_history_field() {
        let stopped = AtomicBool::new(true);
        let mut retained = None;
        assert_eq!(
            generate(
                &mut retained,
                |_| panic!("cancelled before native operation"),
                &stopped
            ),
            Err(DomHostedRuntimeError::Cancelled)
        );
        assert!(retained.is_none());
        let live = AtomicBool::new(false);
        assert_eq!(
            generate(
                &mut retained,
                |bytes| {
                    bytes[..16].fill(255);
                    Err(DomHostedRuntimeError::EntropyFailed)
                },
                &live
            ),
            Err(DomHostedRuntimeError::EntropyFailed)
        );
        assert!(retained.is_none());
    }

    #[test]
    fn successful_fill_retains_both_fields_before_post_fill_cancel_and_retry_reuses_them() {
        let live = AtomicBool::new(false);
        let mut retained = None;
        assert_eq!(
            generate(
                &mut retained,
                |bytes| {
                    bytes.fill(1);
                    live.store(true, Ordering::Relaxed);
                    Ok(())
                },
                &live
            ),
            Err(DomHostedRuntimeError::Cancelled)
        );
        let pair = retained.as_ref().unwrap();
        assert_eq!(
            String::from_utf16(&pair.key).unwrap(),
            "01010101-0101-4101-8101-010101010101"
        );
        assert_eq!(
            String::from_utf16(&pair.id).unwrap(),
            "01010101-0101-4101-8101-010101010101"
        );
        live.store(false, Ordering::Relaxed);
        generate(
            &mut retained,
            |_| panic!("completed fields must not be resampled"),
            &live,
        )
        .unwrap();
        let pair = retained.as_ref().unwrap();
        assert_eq!(
            String::from_utf16(&pair.key).unwrap(),
            "01010101-0101-4101-8101-010101010101"
        );
        assert_eq!(
            String::from_utf16(&pair.id).unwrap(),
            "01010101-0101-4101-8101-010101010101"
        );
    }
}
