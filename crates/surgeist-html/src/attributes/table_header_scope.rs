//! Local th scope keywords and their source-defined content defaults.
//! Table slot anchoring and actual header associations remain table semantics.

use super::*;

/// Conforming authored th scope keywords.
///
/// Group keywords prove local spelling only. Their separate author requirement
/// needs actual row/column group anchoring in a qualified table model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTableHeaderScope {
    Row,
    Column,
    RowGroup,
    ColumnGroup,
}

impl HtmlTableHeaderScope {
    /// Returns the source's conforming canonical author keyword.
    pub fn canonical_keyword(self) -> &'static str {
        match self {
            Self::Row => "row",
            Self::Column => "col",
            Self::RowGroup => "rowgroup",
            Self::ColumnGroup => "colgroup",
        }
    }

    /// Emits a canonical no-namespace scope attribute with ordinary allocation.
    /// This does not prove a th role or the group keywords' anchoring requirement.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            KnownHtmlAttribute::Scope.qualified_name(),
            self.canonical_keyword(),
        )
    }
}

/// Intrinsic th scope state, independent of actual header association.
/// Auto has no conforming authored keyword or native attribute emitter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTableHeaderScopeState {
    Keyword(HtmlTableHeaderScope),
    /// Missing or invalid content selects context-dependent Auto processing.
    Auto,
}

enum_rule!(TH_SCOPE, HtmlTableHeaderScopeState, Scope,
    Context::Elements(&[KnownHtmlElement::Th]), [134], "attr-th-scope",
    Some(HtmlTableHeaderScopeState::Auto), Some(HtmlTableHeaderScopeState::Auto), None,
    ["row" => ("row", HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::Row)),
     "col" => ("col", HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::Column)),
     "rowgroup" => ("rowgroup", HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::RowGroup)),
     "colgroup" => ("colgroup", HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::ColumnGroup))]);

// Borrowed scope processing has the bounded front door, not the report reader's one.
pub(crate) mod bounded_scope {
    use super::*;

    // The bound below counts the ASCII case-insensitive path, which reads pairs
    // only against keywords of the input's own length.
    const _: () = assert!(!TH_SCOPE.exact_keywords);

    /// Returns the th `scope` content state for `raw` without allocating.
    ///
    /// This is the `intrinsic` state of `HtmlValueInterpreter::th_scope` for the
    /// same raw value (HTML 877d §4.9.10, the `scope` enumerated attribute): the
    /// four keywords match ASCII case-insensitively, and an absent, empty or
    /// unrecognized value selects Auto. It builds no report, clones no value, and
    /// makes no element, table or association claim: the caller applies it to a th
    /// element only. The caller admits the raw value's length against its own input
    /// cap and charges [`th_scope_work_bound`] before the call.
    pub fn th_scope_state(raw: Option<&DocumentString>) -> HtmlTableHeaderScopeState {
        match th_scope_state_with(raw, |_| Ok::<_, std::convert::Infallible>(())) {
            Ok(state) => state,
            Err(never) => match never {},
        }
    }

    /// The scope state with each keyword-comparison read admitted just before it.
    fn th_scope_state_with<E>(
        raw: Option<&DocumentString>,
        admit: impl FnMut(bounded_rules::HtmlAttributeKeywordRead) -> Result<(), E>,
    ) -> Result<HtmlTableHeaderScopeState, E> {
        // The rule defines both the missing and invalid defaults as Auto, so every
        // selection carries a state; Auto is that default, not a masked failure.
        Ok(enum_value_selection_with(&TH_SCOPE, raw, admit)?
            .intrinsic
            .unwrap_or(HtmlTableHeaderScopeState::Auto))
    }

    /// Returns the worst-case work units [`th_scope_state`] performs for a raw
    /// value of `input_units` UTF-16 code units, or `None` when it is not
    /// representable.
    ///
    /// One unit is one admitted read (`HtmlAttributeKeywordRead`): the value
    /// header, one header per keyword, and one pair read per raw unit compared
    /// with a keyword spelling. A keyword whose spelling length differs from the
    /// input's costs its header only, so the bound depends on the length only
    /// through the keywords of that length (`row` and `col` at 3, `rowgroup` and
    /// `colgroup` at 8); any other length, however long, costs the headers alone.
    ///
    /// Among the keywords of the input's length, the worst input is a keyword
    /// altered at its last unit: it costs `n` pair reads against that keyword
    /// and, against each other keyword, the ASCII case-insensitive common prefix
    /// of the two spellings plus one mismatching pair (at most `n`). Every input
    /// is bounded by this sum for a keyword `k` it agrees with longest: against
    /// `k` it costs at most `n` pairs, and against any other keyword `j` at most
    /// their common prefix plus one, because an input agreeing with `k` past that
    /// prefix mismatches `j` right after it, and otherwise its agreement with
    /// `j`, no longer than with `k`, ends within the prefix. The bound is
    /// computed from the keyword table, so a changed table stays correct.
    /// Matching earlier or an absent attribute only reduces the work.
    pub fn th_scope_work_bound(input_units: usize) -> Option<usize> {
        let keywords = TH_SCOPE.keywords;
        let mut worst_pairs = 0_usize;
        for (index, keyword) in keywords.iter().enumerate() {
            if keyword.spelling.len() != input_units {
                continue;
            }
            let mut pairs = input_units;
            for (other_index, other) in keywords.iter().enumerate() {
                if other_index == index || other.spelling.len() != input_units {
                    continue;
                }
                let common = keyword
                    .spelling
                    .bytes()
                    .zip(other.spelling.bytes())
                    .take_while(|(a, b)| a.eq_ignore_ascii_case(b))
                    .count();
                pairs = pairs.checked_add(common.saturating_add(1).min(input_units))?;
            }
            worst_pairs = worst_pairs.max(pairs);
        }
        keywords.len().checked_add(1)?.checked_add(worst_pairs)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use HtmlTableHeaderScope::{ColumnGroup, Row, RowGroup};

        const KEYWORDS: [&str; 4] = ["row", "col", "rowgroup", "colgroup"];

        /// HTML 877d §4.9.10, the th scope attribute: the four keywords select
        /// their states, matched ASCII case-insensitively (§2.3.2.2 enumerated
        /// attributes); the missing and invalid value defaults are Auto.
        fn oracle(raw: Option<&[u16]>) -> HtmlTableHeaderScopeState {
            let Some(raw) = raw else {
                return HtmlTableHeaderScopeState::Auto;
            };
            let Ok(text) = String::from_utf16(raw) else {
                return HtmlTableHeaderScopeState::Auto;
            };
            // Only ASCII letters fold: Kelvin sign and long s are different units.
            let lowered: String = text
                .chars()
                .map(|c| {
                    if c.is_ascii() {
                        c.to_ascii_lowercase()
                    } else {
                        c
                    }
                })
                .collect();
            match lowered.as_str() {
                "row" => HtmlTableHeaderScopeState::Keyword(Row),
                "col" => HtmlTableHeaderScopeState::Keyword(HtmlTableHeaderScope::Column),
                "rowgroup" => HtmlTableHeaderScopeState::Keyword(RowGroup),
                "colgroup" => HtmlTableHeaderScopeState::Keyword(ColumnGroup),
                _ => HtmlTableHeaderScopeState::Auto,
            }
        }

        fn counted(raw: Option<&DocumentString>) -> (HtmlTableHeaderScopeState, usize) {
            let mut count = 0;
            let state = th_scope_state_with(raw, |_| {
                count += 1;
                Ok::<_, std::convert::Infallible>(())
            })
            .unwrap_or_else(|never| match never {});
            (state, count)
        }

        /// Keyword spellings in every ASCII case pattern of every prefix length
        /// up to two past the end, every single-unit substitution, long runs, and
        /// the non-keyword values the Standard's tests use.
        fn corpus() -> Vec<Vec<u16>> {
            let mut all: Vec<Vec<u16>> = vec![Vec::new()];
            for keyword in KEYWORDS {
                let units: Vec<u16> = keyword.encode_utf16().collect();
                for mask in 0..(1_u32 << units.len()) {
                    let cased: Vec<u16> = units
                        .iter()
                        .enumerate()
                        .map(|(i, &u)| if mask >> i & 1 == 1 { u - 32 } else { u })
                        .collect();
                    for length in 0..=cased.len() {
                        all.push(cased[..length].to_vec());
                    }
                    for tail in *b"x 0" {
                        let mut extended = cased.clone();
                        extended.push(u16::from(tail));
                        all.push(extended);
                    }
                    for position in 0..cased.len() {
                        for substitute in [u16::from(b'x'), 0x212a, 0x17f, 0x0] {
                            let mut changed = cased.clone();
                            changed[position] = substitute;
                            all.push(changed);
                        }
                    }
                }
            }
            for length in [1_usize, 2, 3, 8, 9, 1000, 100_000] {
                all.push(vec![u16::from(b'r'); length]);
                all.push(vec![u16::from(b'c'); length]);
            }
            for other in ["auto", "AUTO", " row", "row ", "rows", "", "r\u{ff4f}w"] {
                all.push(other.encode_utf16().collect());
            }
            all
        }

        #[test]
        fn states_match_the_standard_and_the_report_reader() {
            assert_eq!(th_scope_state(None), HtmlTableHeaderScopeState::Auto);
            let interpreter = HtmlValueInterpreter::new(1_000_000);
            let th = KnownHtmlElement::Th.qualified_name();
            for units in corpus() {
                let raw = DocumentString::from_utf16(units.clone());
                let attribute =
                    Attribute::new(KnownHtmlAttribute::Scope.qualified_name(), raw.clone());
                let state = th_scope_state(Some(&raw));
                assert_eq!(state, oracle(Some(&units)), "{units:?}");
                assert_eq!(
                    interpreter
                        .th_scope(&th, Some(&attribute))
                        .unwrap()
                        .intrinsic()
                        .copied(),
                    Some(state),
                    "{units:?}"
                );
            }
        }

        /// Work per input length, derived by hand from the keyword list
        /// (`row`, `col`, `rowgroup`, `colgroup`) and the comparison rule: the
        /// value header and four keyword headers cost 5 for every input. A unit
        /// pair is read only against a keyword of the input's own length, so
        /// only lengths 3 and 8 add any. In each of those two lengths the two
        /// keywords differ at their first unit (`r` and `c`), so the worst input
        /// agrees with one keyword in every unit but the last (n pairs against
        /// it) and costs one first-unit pair against the other: 5 + 3 + 1 = 9
        /// and 5 + 8 + 1 = 14.
        fn hand_derived_worst(units: usize) -> usize {
            match units {
                3 => 9,
                8 => 14,
                _ => 5,
            }
        }

        #[test]
        fn work_never_exceeds_the_published_bound_and_the_bound_is_reached() {
            assert_eq!(counted(None), (HtmlTableHeaderScopeState::Auto, 1));
            let mut worst = std::collections::BTreeMap::new();
            for units in corpus() {
                let raw = DocumentString::from_utf16(units.clone());
                let (_, work) = counted(Some(&raw));
                let bound = th_scope_work_bound(units.len()).unwrap();
                assert!(work <= bound, "{units:?}: {work} units, bound {bound}");
                let entry = worst.entry(units.len()).or_insert(0);
                *entry = work.max(*entry);
            }
            // The bound is the worst case for its length, reached by some input
            // in the corpus at every length it covers, including long values
            // whose work is the constant five.
            for (length, work) in worst {
                assert_eq!(work, hand_derived_worst(length), "worst work at {length}");
                assert_eq!(th_scope_work_bound(length), Some(work), "bound at {length}");
            }
            for length in [0_usize, 1, 2, 4, 5, 6, 7, 9, 10, 1000, 100_000, usize::MAX] {
                assert_eq!(th_scope_work_bound(length), Some(5), "bound at {length}");
            }
            let long = DocumentString::from_utf16(vec![u16::from(b'r'); 100_000]);
            assert_eq!(counted(Some(&long)).1, 5);
            assert_eq!(th_scope_work_bound(3), Some(9));
            assert_eq!(th_scope_work_bound(8), Some(14));
        }
    }
}

impl HtmlValueInterpreter {
    /// Interprets th scope content through the shared enumeration owner.
    ///
    /// Missing and invalid content select Auto without changing recognition or
    /// author spelling. Empty and literal `auto` are invalid authored keywords.
    /// Selected RowGroup/ColumnGroup retain their state and leave actual table
    /// anchoring Unchecked; other states select no additional context check.
    /// No parent-name inference, table reconstruction, header association or
    /// complete th/table conformance is performed.
    ///
    /// Entire original raw UTF-16 admission precedes expanded-name applicability.
    /// Reports use ordinary shared allocation and the existing per-value cap,
    /// without physical-memory, work or cancellation accounting.
    pub fn th_scope(
        &self,
        element: &(impl crate::HtmlNameInput + ?Sized),
        raw: Option<&impl crate::HtmlAttributeInput>,
    ) -> Result<HtmlAttributeReport<HtmlTableHeaderScopeState>, HtmlValueAdmissionError> {
        let mut report = self.enumerated(&TH_SCOPE, element, raw)?;
        if matches!(
            report.intrinsic,
            Some(HtmlTableHeaderScopeState::Keyword(
                HtmlTableHeaderScope::RowGroup | HtmlTableHeaderScope::ColumnGroup
            ))
        ) {
            report.context_author = HtmlAttributeContextAuthorState::Unchecked;
        }
        Ok(report)
    }
}
