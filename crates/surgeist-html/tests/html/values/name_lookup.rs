//! Recognition of the closed HTML element and event handler vocabularies.
//!
//! The expectations come from the vocabulary table (`ALL` and the canonical
//! spellings) through two independent references: a scan of the table, and a
//! binary search over a copy of the table sorted by the UTF-16 units of its
//! spellings.

use surgeist_html::{
    values::{DocumentString, Namespace, QualifiedName},
    vocabulary::{KnownHtmlElement, KnownHtmlEventHandler},
};

/// Units that replace, or are inserted into, a spelling: ASCII letters and
/// punctuation of the vocabulary, case counterparts, controls, the edges of the
/// non-ASCII range and lone surrogates.
const PROBE_UNITS: [u16; 14] = [
    0x00, 0x2D, 0x30, 0x41, 0x5A, 0x61, 0x7A, 0x7B, 0xE9, 0xD7FF, 0xD800, 0xDBFF, 0xDC00, 0xFFFF,
];

fn spelling<T>(known: T, local_name: fn(T) -> &'static str) -> Vec<u16> {
    local_name(known).encode_utf16().collect()
}

/// The table scan and the sorted-table binary search, which must agree.
struct Reference<T: Copy + PartialEq + std::fmt::Debug + 'static> {
    all: &'static [T],
    local_name: fn(T) -> &'static str,
    sorted: Vec<T>,
}

impl<T: Copy + PartialEq + std::fmt::Debug + 'static> Reference<T> {
    fn new(all: &'static [T], local_name: fn(T) -> &'static str) -> Self {
        let mut sorted = all.to_vec();
        sorted.sort_by(|left, right| {
            local_name(*left)
                .encode_utf16()
                .cmp(local_name(*right).encode_utf16())
        });
        Self {
            all,
            local_name,
            sorted,
        }
    }

    fn expected(&self, units: &[u16]) -> Option<T> {
        let by_scan = self.all.iter().copied().find(|known| {
            (self.local_name)(*known)
                .encode_utf16()
                .eq(units.iter().copied())
        });
        let by_search = self
            .sorted
            .binary_search_by(|known| {
                (self.local_name)(*known)
                    .encode_utf16()
                    .cmp(units.iter().copied())
            })
            .ok()
            .and_then(|index| self.sorted.get(index).copied());
        assert_eq!(by_scan, by_search, "references disagree on {units:?}");
        by_scan
    }

    /// Every name, every name with one unit changed, inserted or removed, every
    /// ASCII case variant, non-ASCII and lone-surrogate inputs, the empty name
    /// and a seeded set of random strings.
    fn probes(&self) -> Vec<Vec<u16>> {
        let mut probes: Vec<Vec<u16>> = vec![
            Vec::new(),
            vec![0xD800],
            vec![0xDC00],
            vec![0x61, 0xDFFF],
            vec![0xD83D, 0xDE00],
            "é".encode_utf16().collect(),
            vec![0; 3],
            vec![0x61; 300],
        ];
        for known in self.all {
            let units = spelling(*known, self.local_name);
            probes.push(units.clone());
            let mut shorter = units.clone();
            shorter.pop();
            probes.push(shorter);
            for at in 0..units.len() {
                let mut removed = units.clone();
                removed.remove(at);
                probes.push(removed);
                for probe in PROBE_UNITS {
                    let mut changed = units.clone();
                    changed[at] = probe;
                    probes.push(changed);
                }
                let mut next = units.clone();
                next[at] = next[at].wrapping_add(1);
                probes.push(next);
                let mut previous = units.clone();
                previous[at] = previous[at].wrapping_sub(1);
                probes.push(previous);
            }
            for at in 0..=units.len() {
                for probe in PROBE_UNITS {
                    let mut inserted = units.clone();
                    inserted.insert(at, probe);
                    probes.push(inserted);
                }
            }
            for at in 0..units.len() {
                let mut folded = units.clone();
                if let Some(unit) = folded.get_mut(at)
                    && matches!(*unit, 0x61..=0x7A)
                {
                    *unit -= 0x20;
                }
                probes.push(folded);
            }
            probes.push(
                units
                    .iter()
                    .map(|unit| match *unit {
                        0x61..=0x7A => unit - 0x20,
                        other => other,
                    })
                    .collect(),
            );
            probes.push([units.as_slice(), units.as_slice()].concat());
        }
        let alphabet: Vec<u16> = self
            .all
            .iter()
            .flat_map(|known| spelling(*known, self.local_name))
            .collect();
        let mut state: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = move || {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            state.wrapping_mul(0x2545_F491_4F6C_DD1D)
        };
        for _ in 0..20_000 {
            let length = (next() % 24) as usize;
            let random: Vec<u16> = (0..length)
                .map(|_| {
                    let draw = next();
                    if draw % 4 == 0 {
                        (draw >> 8) as u16
                    } else {
                        alphabet[(draw >> 8) as usize % alphabet.len()]
                    }
                })
                .collect();
            probes.push(random);
        }
        probes
    }
}

fn html_name(units: &[u16]) -> QualifiedName {
    QualifiedName::new(Namespace::html(), None, DocumentString::from_utf16(units))
}

#[test]
fn html_element_recognition_equals_the_vocabulary_table_over_every_probe() {
    let reference = Reference::new(KnownHtmlElement::ALL, KnownHtmlElement::local_name);
    let mut recognized = 0;
    for units in reference.probes() {
        let expected = reference.expected(&units);
        recognized += usize::from(expected.is_some());
        assert_eq!(
            KnownHtmlElement::from_name(&html_name(&units)),
            expected,
            "{units:?}"
        );
    }
    // Every spelling is a probe, and the neighbours of each are mostly misses.
    assert!(recognized >= KnownHtmlElement::ALL.len());
}

#[test]
fn html_element_recognition_requires_the_html_namespace_for_every_known_name() {
    for known in KnownHtmlElement::ALL {
        let units = spelling(*known, KnownHtmlElement::local_name);
        for namespace in [
            Namespace::none(),
            Namespace::svg(),
            Namespace::mathml(),
            Namespace::from_uri("urn:other"),
        ] {
            let name = QualifiedName::new(
                namespace,
                None,
                DocumentString::from_utf16(units.as_slice()),
            );
            assert_eq!(KnownHtmlElement::from_name(&name), None, "{known:?}");
        }
        assert_eq!(
            KnownHtmlElement::from_name(&html_name(&units)),
            Some(*known)
        );
    }
}

#[test]
fn event_handler_recognition_equals_the_vocabulary_table_over_every_probe() {
    let reference = Reference::new(
        KnownHtmlEventHandler::ALL,
        KnownHtmlEventHandler::local_name,
    );
    let mut recognized = 0;
    for units in reference.probes() {
        let expected = reference.expected(&units);
        recognized += usize::from(expected.is_some());
        assert_eq!(
            KnownHtmlEventHandler::from_local_units(&units),
            expected,
            "{units:?}"
        );
        let name = QualifiedName::new(
            Namespace::none(),
            None,
            DocumentString::from_utf16(units.as_slice()),
        );
        assert_eq!(
            KnownHtmlEventHandler::from_name(&name),
            expected,
            "{units:?}"
        );
        assert_eq!(
            KnownHtmlEventHandler::from_name(&html_name(&units)),
            None,
            "{units:?}"
        );
    }
    assert!(recognized >= KnownHtmlEventHandler::ALL.len());
}
