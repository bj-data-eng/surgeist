//! CSS strictness adaptation around the pinned ICU line-break provider.
//!
//! Policies share complete semantic components, never truncated style runs.
//! Anywhere's separate text/TCU opportunity owner lives in protocol::prepared.

use std::ops::Range;

use icu_properties::{
    CodePointMapData,
    props::{EastAsianWidth, GeneralCategory, LineBreak as Class},
};
use icu_segmenter::{
    LineSegmenter,
    options::{LineBreakOptions, LineBreakStrictness, LineBreakWordOption},
};

use super::{Boundary, is_hard_separator, style_at};
use crate::{
    LineBreak, engine::resolve::AnalyzedStyleRun, projection::SourceProjection,
    style::OrdinaryWordBreak,
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct Policy {
    strictness: LineBreak,
    word: OrdinaryWordBreak,
    cjk: bool,
}

impl Policy {
    const BASELINE: Self = Self {
        strictness: LineBreak::Strict,
        word: OrdinaryWordBreak::Normal,
        cjk: false,
    };

    fn normal_word(self) -> Self {
        Self {
            word: OrdinaryWordBreak::Normal,
            ..self
        }
    }
}

pub(super) fn policies(styles: &[AnalyzedStyleRun]) -> crate::Result<Vec<Policy>> {
    styles
        .iter()
        .map(|row| {
            let style = row.style.authored();
            let breaking = style.effective_breaking();
            let strictness = match style.line_break {
                LineBreak::Auto | LineBreak::Anywhere => LineBreak::Strict,
                other => other,
            };
            let cjk = if let Some(locale) = row.style.font_locale() {
                // Parse only the already admitted canonical language/script/region
                // prefix. Extensions remain untouched in the authored Style.
                let mut id: icu_locale::LanguageIdentifier =
                    locale.as_str().parse().map_err(|_| {
                        crate::Error::new(
                            crate::ErrorCode::InvalidStyle,
                            "canonical content locale cannot be represented for line breaking",
                        )
                    })?;
                if id.script.is_none() && id.language.as_str() != "und" {
                    icu_locale::LocaleExpander::new_extended().maximize(&mut id);
                }
                id.script.is_some_and(|script| {
                    matches!(
                        script.as_str(),
                        "Hant"
                            | "Hans"
                            | "Hani"
                            | "Hanb"
                            | "Bopo"
                            | "Jpan"
                            | "Hrkt"
                            | "Hira"
                            | "Kana"
                    )
                })
            } else {
                false
            };
            Ok(Policy {
                strictness,
                word: breaking.word,
                cjk: cjk && strictness != LineBreak::Strict,
            })
        })
        .collect()
}

/// Constructor-bound absence, separate from the cut result (which may be empty).
/// Only this concrete provider selection can construct the witness.
pub(super) struct MissingLexicalModels(());

/// Only the line provider sees these class representatives. Every endpoint
/// maps back to the unchanged component's original UTF-8 byte boundary.
enum ProviderInput<'a> {
    Original(&'a str),
    ResolvedSa {
        text: String,
        boundaries: Vec<(usize, usize)>,
        original_len: usize,
    },
}

impl<'a> ProviderInput<'a> {
    fn without_lexical_models(original: &'a str) -> Self {
        let classes = CodePointMapData::<Class>::new();
        if !original
            .chars()
            .any(|ch| classes.get(ch) == Class::ComplexContext)
        {
            return Self::Original(original);
        }
        let mut text = String::with_capacity(original.len());
        let mut boundaries = Vec::new();
        for (byte, ch) in original.char_indices() {
            boundaries.push((text.len(), byte));
            let class = classes.get(ch);
            text.push(if class == Class::ComplexContext {
                match resolved_class(ch as u32, class) {
                    Class::CombiningMark => '\u{300}',
                    _ => 'a',
                }
            } else {
                ch
            });
        }
        boundaries.push((text.len(), original.len()));
        Self::ResolvedSa {
            text,
            boundaries,
            original_len: original.len(),
        }
    }

    fn text(&self) -> &str {
        match self {
            Self::Original(text) => text,
            Self::ResolvedSa { text, .. } => text,
        }
    }
}

fn resolved_class(code_point: u32, class: Class) -> Class {
    if class == Class::ComplexContext {
        // Without lexical models, ordinary SA uses LB1's AL/CM context;
        // qualified lexical fallback is proposed separately from these cuts.
        if matches!(
            CodePointMapData::<GeneralCategory>::new().get32(code_point),
            GeneralCategory::NonspacingMark | GeneralCategory::SpacingMark
        ) {
            Class::CombiningMark
        } else {
            Class::Alphabetic
        }
    } else {
        class
    }
}

fn provider(input: &ProviderInput<'_>, policy: Policy) -> (Vec<bool>, MissingLexicalModels) {
    let mut options = LineBreakOptions::default();
    options.strictness = Some(match policy.strictness {
        LineBreak::Loose => LineBreakStrictness::Loose,
        LineBreak::Normal => LineBreakStrictness::Normal,
        _ => LineBreakStrictness::Strict,
    });
    options.word_option = Some(match policy.word {
        OrdinaryWordBreak::Normal => LineBreakWordOption::Normal,
        OrdinaryWordBreak::BreakAll => LineBreakWordOption::BreakAll,
        OrdinaryWordBreak::KeepAll => LineBreakWordOption::KeepAll,
    });
    // ICU's only content-locale distinction is ja/zh. This identifier conveys
    // the proven Appendix-F classification, never an invented authored locale.
    let cjk = icu_locale::langid!("ja");
    options.content_locale = policy.cjk.then_some(&cjk);
    let original_len = match input {
        ProviderInput::Original(text) => text.len(),
        ProviderInput::ResolvedSa { original_len, .. } => *original_len,
    };
    let mut cuts = vec![false; original_len + 1];
    let mut cursor = 0;
    for byte in LineSegmenter::new_for_non_complex_scripts(options).segment_str(input.text()) {
        let original = match input {
            ProviderInput::Original(_) => byte,
            ProviderInput::ResolvedSa { boundaries, .. } => {
                while boundaries[cursor].0 < byte {
                    cursor += 1;
                }
                // ICU issues scalar endpoints in order; the complete map also
                // contains start/end sentinels and repeated endpoints are safe.
                assert_eq!(boundaries[cursor].0, byte);
                boundaries[cursor].1
            }
        };
        cuts[original] = true;
    }
    (cuts, MissingLexicalModels(()))
}

pub(super) fn apply(
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
    policies: &[Policy],
    offsets: &[usize],
    component: Range<usize>,
    boundaries: &mut [Boundary],
) -> MissingLexicalModels {
    let mut needed = Vec::new();
    for atom in component.clone() {
        let policy = policies[style_at(projection, styles, atom)];
        if !needed.contains(&policy) {
            needed.push(policy);
        }
    }
    // At most 3 strictnesses * 3 word modes * 2 CJK classifications.
    let start = offsets[component.start];
    let text = &projection.segmentation[start..offsets[component.end]];
    // This input resolution belongs to the selected non-complex constructor,
    // not to a style mode or the presence/absence of returned cut positions.
    let input = ProviderInput::without_lexical_models(text);
    let (baseline, missing_lexical) = provider(&input, Policy::BASELINE);
    let classes = CodePointMapData::<Class>::new();
    let mut effective = Vec::with_capacity(component.len());
    let mut owners = Vec::with_capacity(component.len());
    let mut opening = Vec::with_capacity(component.len());
    let mut zero_width_space = Vec::with_capacity(component.len());
    let mut blocks_opening = false;
    let mut follows_zero_width_space = false;
    for atom in component.clone() {
        let code_point = projection.atoms[atom].code_point;
        let class = resolved_class(code_point, classes.get32(code_point));
        let inherited = matches!(class, Class::CombiningMark | Class::ZWJ)
            .then(|| {
                effective.last().copied().filter(|previous| {
                    !matches!(
                        *previous,
                        Class::MandatoryBreak
                            | Class::CarriageReturn
                            | Class::LineFeed
                            | Class::NextLine
                            | Class::Space
                            | Class::ZWSpace
                    )
                })
            })
            .flatten();
        let local = atom - component.start;
        let owner = if inherited.is_some() {
            owners[local - 1]
        } else {
            local
        };
        let class = inherited.unwrap_or({
            if matches!(class, Class::CombiningMark | Class::ZWJ) {
                Class::Alphabetic
            } else {
                class
            }
        });
        owners.push(owner);
        effective.push(class);
        if class != Class::Space {
            follows_zero_width_space = class == Class::ZWSpace;
            let cp = projection.atoms[component.start + owner].code_point;
            let prior = owner.checked_sub(1).map(|i| effective[i]);
            blocks_opening = class == Class::OpenPunctuation
                || class == Class::Quotation
                    && CodePointMapData::<GeneralCategory>::new().get32(cp)
                        == GeneralCategory::InitialPunctuation
                    && prior.is_none_or(|value| {
                        matches!(
                            value,
                            Class::MandatoryBreak
                                | Class::CarriageReturn
                                | Class::LineFeed
                                | Class::NextLine
                                | Class::OpenPunctuation
                                | Class::Quotation
                                | Class::Glue
                                | Class::Space
                                | Class::ZWSpace
                        )
                    });
        }
        opening.push(blocks_opening);
        zero_width_space.push(follows_zero_width_space);
    }
    // Provider local0/eot are sentinels, never text/text permission. Actual
    // terminal ZW/SP causes retain their independently qualified permissions.
    if independent_ending(projection, styles, &component, &effective, &opening) {
        boundaries[component.end] = Boundary::Line;
    }
    let mut groups = Vec::new();
    for policy in &needed {
        if !groups.contains(&policy.normal_word()) {
            groups.push(policy.normal_word());
        }
    }
    // Retain only baseline, one strictness/locale Normal-word pass, and one
    // actual word-mode pass at a time. Each finite provider tuple runs once.
    for group in groups {
        let normal_storage = (group != Policy::BASELINE).then(|| provider(&input, group).0);
        let normal = normal_storage.as_ref().unwrap_or(&baseline);
        for policy in needed
            .iter()
            .copied()
            .filter(|policy| policy.normal_word() == group)
        {
            let actual_storage =
                (policy.word != OrdinaryWordBreak::Normal).then(|| provider(&input, policy).0);
            let actual = actual_storage.as_ref().unwrap_or(normal);
            for atom in component.start + 1..component.end {
                if policies[style_at(projection, styles, atom)] != policy {
                    continue;
                }
                let byte = offsets[atom] - start;
                let independent = baseline[byte];
                let before = effective[atom - component.start - 1];
                let after = classes.get32(projection.atoms[atom].code_point);
                let required = required_pair(
                    projection.atoms[atom - 1].code_point,
                    projection.atoms[atom].code_point,
                    before,
                    after,
                    policy,
                );
                let allowed = match required {
                    // Strict Normal-word output preserves independently caused SPACE
                    // and ZWSP permissions without retaining the provider's extra CJ.
                    Some(false) => independent,
                    Some(true) => actual[byte] || normal[byte],
                    None => actual[byte],
                };
                if !allowed
                    || !provider_cut_allowed(
                        projection.atoms[atom - 1].code_point,
                        projection.atoms[atom].code_point,
                        before,
                        after,
                        zero_width_space[atom - component.start - 1],
                    )
                {
                    continue;
                }
                if !independent {
                    // Retain the extra-tailoring floor before ZWJ. Baseline
                    // before-ZWJ permissions remain provider-owned: LB8a is an
                    // after-ZWJ prohibition, and LB9 excludes prior SP/hard/ZW.
                    if after == Class::ZWJ {
                        continue;
                    }
                    // Preserve UAX55 LB14/LB15a and quote rules bypassed by ICU's
                    // early CSS return. Opening state is precomputed, not rescanned.
                    if opening[atom - component.start - 1]
                        || quote_blocks(projection, &component, &effective, &owners, atom)
                    {
                        continue;
                    }
                }
                boundaries[atom] = Boundary::Line;
            }
        }
    }
    for (atom, boundary) in boundaries
        .iter_mut()
        .enumerate()
        .take(component.end)
        .skip(component.start + 1)
    {
        if [atom - 1, atom].into_iter().any(|at| {
            styles[style_at(projection, styles, at)]
                .style
                .authored()
                .line_break
                == LineBreak::Anywhere
        }) && !is_hard_separator(projection.atoms[atom - 1].code_point)
            && !is_hard_separator(projection.atoms[atom].code_point)
            && projection
                .graphemes
                .binary_search(&projection.atoms[atom].presentation.start)
                .is_ok()
            && *boundary < Boundary::Line
        {
            *boundary = Boundary::Anywhere;
        }
    }
    missing_lexical
}

fn provider_cut_allowed(
    before_code_point: u32,
    after_code_point: u32,
    before: Class,
    after: Class,
    follows_zero_width_space: bool,
) -> bool {
    // Provider cuts and ICU's early CSS returns still need actual semantic
    // protection, including inherited glue. Baseline cuts are no exemption.
    // Semantic hard separators and LB7 precede LB8's component-local ZW SP*.
    if is_hard_separator(before_code_point)
        || is_hard_separator(after_code_point)
        || matches!(after, Class::Space | Class::ZWSpace)
    {
        return false;
    }
    if follows_zero_width_space {
        return true;
    }
    // LB8a/LB11/LB12, followed by LB12a's explicit before-GL exceptions.
    !matches!(before, Class::WordJoiner | Class::Glue)
        && after != Class::WordJoiner
        && before_code_point != 0x200d
        && (after != Class::Glue
            || matches!(
                before,
                Class::Space | Class::BreakAfter | Class::Hyphen | Class::UnambiguousHyphen
            ))
}

fn independent_ending(
    projection: &SourceProjection,
    styles: &[AnalyzedStyleRun],
    component: &Range<usize>,
    effective: &[Class],
    opening: &[bool],
) -> bool {
    let end = component.end;
    let Some(after) = projection.atoms.get(end) else {
        return false;
    };
    let classes = CodePointMapData::<Class>::new();
    let right = classes.get32(after.code_point);
    // UAX55 LB6/LB7 precede LB8. Inspect only the actual retained suffix of
    // this component; another component can never supply its ZW authority.
    if is_hard_separator(after.code_point) || matches!(right, Class::Space | Class::ZWSpace) {
        return false;
    }
    let prior = effective
        .iter()
        .rev()
        .find(|class| **class != Class::Space)
        .copied();
    // LB8's ZW SP* permission precedes the WJ/glue/punctuation prohibitions
    // below. Prepared TCU, mandatory and wrapping guards still own admission.
    if prior == Some(Class::ZWSpace) {
        return true;
    }
    let before = &projection.atoms[end - 1];
    let style = styles[style_at(projection, styles, end - 1)]
        .style
        .authored();
    if !matches!(before.code_point, 0x20 | 0x09)
        || !matches!(
            style.white_space,
            crate::WhiteSpace::Preserve | crate::WhiteSpace::PreserveSpaces
        )
        || matches!(after.code_point, 0x20 | 0x09)
        || opening.last() == Some(&true)
    {
        return false;
    }
    // Incident glue/hard rules and pre-SPACE opening context remain relevant
    // to this whitespace owner; no Unicode text pair crosses the component.
    if matches!(
        right,
        Class::WordJoiner
            | Class::ZWJ
            | Class::ZWSpace
            | Class::ClosePunctuation
            | Class::CloseParenthesis
            | Class::Exclamation
            | Class::BreakSymbols
    ) {
        return false;
    }
    if matches!(
        prior,
        Some(Class::ClosePunctuation | Class::CloseParenthesis)
    ) && matches!(right, Class::Nonstarter | Class::ConditionalJapaneseStarter)
        || prior == Some(Class::BreakBoth) && right == Class::BreakBoth
    {
        return false;
    }
    let next = projection
        .atoms
        .get(end + 1)
        .map(|atom| classes.get32(atom.code_point));
    if right == Class::InfixNumeric && next != Some(Class::Numeric) {
        return false;
    }
    if right == Class::Quotation
        && CodePointMapData::<GeneralCategory>::new().get32(after.code_point)
            == GeneralCategory::FinalPunctuation
        && next.is_none_or(|class| {
            matches!(
                class,
                Class::Space
                    | Class::Glue
                    | Class::WordJoiner
                    | Class::ClosePunctuation
                    | Class::Quotation
                    | Class::CloseParenthesis
                    | Class::Exclamation
                    | Class::InfixNumeric
                    | Class::BreakSymbols
                    | Class::MandatoryBreak
                    | Class::CarriageReturn
                    | Class::LineFeed
                    | Class::NextLine
                    | Class::ZWSpace
            )
        })
    {
        return false;
    }
    true
}

fn quote_blocks(
    projection: &SourceProjection,
    component: &Range<usize>,
    effective: &[Class],
    owners: &[usize],
    atom: usize,
) -> bool {
    let local = atom - component.start;
    let before = owners[local - 1];
    let cp = |i: usize| projection.atoms[component.start + i].code_point;
    let east_asian = |i| {
        matches!(
            CodePointMapData::<EastAsianWidth>::new().get32(cp(i)),
            EastAsianWidth::Fullwidth | EastAsianWidth::Wide | EastAsianWidth::Halfwidth
        )
    };
    let category = |i| CodePointMapData::<GeneralCategory>::new().get32(cp(i));
    if effective[local - 1] == Class::Quotation
        && (category(before) != GeneralCategory::FinalPunctuation
            || before.checked_sub(1).is_none_or(|i| !east_asian(i))
            || !east_asian(local))
    {
        return true;
    }
    if effective[local] != Class::Quotation {
        return false;
    }
    let mut next = local + 1;
    while next < owners.len() && owners[next] == local {
        next += 1;
    }
    // LB15b, LB19 and LB19a. These are contextual quote restrictions, not a
    // blanket veto of all quotation marks or all East Asian punctuation.
    category(local) != GeneralCategory::InitialPunctuation
        || !east_asian(local - 1)
        || next == owners.len()
        || !east_asian(next)
        || category(local) == GeneralCategory::FinalPunctuation
            && matches!(
                effective[next],
                Class::Space
                    | Class::Glue
                    | Class::WordJoiner
                    | Class::ClosePunctuation
                    | Class::Quotation
                    | Class::CloseParenthesis
                    | Class::Exclamation
                    | Class::InfixNumeric
                    | Class::BreakSymbols
                    | Class::MandatoryBreak
                    | Class::CarriageReturn
                    | Class::LineFeed
                    | Class::NextLine
                    | Class::ZWSpace
            )
}

/// The bounded mandatory CSS distinctions; all other UAX behavior stays ICU's.
fn required_pair(
    before_cp: u32,
    after_cp: u32,
    before: Class,
    after: Class,
    policy: Policy,
) -> Option<bool> {
    let loose = policy.strictness == LineBreak::Loose;
    if matches!(after_cp, 0x301c | 0x30a0) {
        return Some(policy.strictness != LineBreak::Strict && policy.cjk);
    }
    if matches!(after_cp, 0x2010 | 0x2013) {
        let id = before == Class::Ideographic
            || before == Class::ConditionalJapaneseStarter && loose
            || policy.word == OrdinaryWordBreak::BreakAll
                && matches!(
                    before,
                    Class::Alphabetic | Class::Numeric | Class::ComplexContext | Class::Ambiguous
                );
        return Some(loose && id);
    }
    if matches!(
        after_cp,
        0x3005 | 0x303b | 0x309d | 0x309e | 0x30fd | 0x30fe
    ) || after == Class::Inseparable
    {
        return Some(loose);
    }
    // BreakAll converts processed CJ letter units to effective ID before this
    // class-based restriction. Named iteration/punctuation rules above retain
    // their independent precedence (see the adopted small-kana determination).
    if after == Class::ConditionalJapaneseStarter {
        return (policy.word != OrdinaryWordBreak::BreakAll).then_some(loose);
    }
    if matches!(
        after_cp,
        0x30fb | 0xff1a | 0xff1b | 0xff65 | 0x203c | 0x2047..=0x2049 | 0xff01 | 0xff1f
    ) {
        return Some(loose && policy.cjk);
    }
    let east_asian = |cp| {
        matches!(
            CodePointMapData::<EastAsianWidth>::new().get32(cp),
            EastAsianWidth::Ambiguous | EastAsianWidth::Fullwidth | EastAsianWidth::Wide
        )
    };
    if after == Class::PostfixNumeric && east_asian(after_cp)
        || before == Class::PrefixNumeric && east_asian(before_cp)
    {
        return Some(loose && policy.cjk);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ordinary_boundaries(
        text: &str,
        word_break: crate::WordBreak,
        line_break: LineBreak,
    ) -> Vec<Boundary> {
        let source = crate::Source::new(text);
        let projection = SourceProjection::new(&source).unwrap();
        let styles = vec![AnalyzedStyleRun {
            owner: crate::engine::resolve::AnalyzedStyleOwner::Source(crate::Range::new(
                0,
                source.text().len(),
            )),
            style: crate::ValidatedStyle::try_from(crate::Style {
                word_break,
                line_break,
                ..crate::Style::default()
            })
            .unwrap(),
        }];
        let offsets: Vec<_> = projection
            .segmentation
            .char_indices()
            .map(|(byte, _)| byte)
            .chain(std::iter::once(projection.segmentation.len()))
            .collect();
        let mut boundaries = vec![Boundary::None; projection.atoms.len() + 1];
        apply(
            &projection,
            &styles,
            &policies(&styles).unwrap(),
            &offsets,
            0..projection.atoms.len(),
            &mut boundaries,
        );
        boundaries
    }

    #[test]
    fn ordinary_provider_resolves_sa_marks_and_maps_cuts_to_original_multibyte_endpoints() {
        let classes = CodePointMapData::<Class>::new();
        assert_eq!(classes.get('a'), Class::Alphabetic);
        assert_eq!(classes.get('\u{300}'), Class::CombiningMark);
        assert_eq!(classes.get('\u{10400}'), Class::Alphabetic);
        assert_eq!(classes.get('\u{e48}'), Class::ComplexContext);
        assert_eq!(
            CodePointMapData::<GeneralCategory>::new().get('\u{e48}'),
            GeneralCategory::NonspacingMark
        );
        let text = "\u{10400}ก\u{e48}ก A";
        let input = ProviderInput::without_lexical_models(text);
        let (cuts, _) = provider(&input, Policy::BASELINE);
        // AL CM AL remains one alphabetic run; LB18 supplies the SPACE cut.
        // The untouched supplementary AL and shorter representatives require
        // actual endpoint mapping, not a constant byte displacement.
        assert_eq!(
            cuts.iter()
                .enumerate()
                .filter_map(|(byte, cut)| cut.then_some(byte))
                .collect::<Vec<_>>(),
            [0, "\u{10400}ก\u{e48}ก ".len(), text.len()]
        );
    }

    #[test]
    fn non_sa_input_borrows_original_storage_and_keeps_provider_cuts() {
        let text = "abc A";
        let input = ProviderInput::without_lexical_models(text);
        assert!(std::ptr::eq(input.text().as_ptr(), text.as_ptr()));
        let (cuts, _) = provider(&input, Policy::BASELINE);
        assert_eq!(
            cuts.iter()
                .enumerate()
                .filter_map(|(byte, cut)| cut.then_some(byte))
                .collect::<Vec<_>>(),
            [0, 4, 5]
        );
    }

    #[test]
    fn ordinary_baseline_cannot_restore_sa_run_end_before_strict_small_kana() {
        for word_break in [
            crate::WordBreak::Normal,
            crate::WordBreak::KeepAll,
            crate::WordBreak::BreakWord,
            crate::WordBreak::Manual,
        ] {
            for strictness in [LineBreak::Auto, LineBreak::Normal, LineBreak::Strict] {
                assert_ne!(
                    ordinary_boundaries("กกぁ", word_break, strictness)[2],
                    Boundary::Line,
                    "{strictness:?}"
                );
            }
            assert_eq!(
                ordinary_boundaries("กกぁ", word_break, LineBreak::Loose)[2],
                Boundary::Line
            );
        }
    }

    #[test]
    fn ordinary_sa_resolution_retains_full_al_numeric_and_opening_context() {
        // UAX55 LB23/28/29 retain alphabetic/numeric words; LB14 retains
        // opening context across spaces. LB18's unrelated SPACE cut survives.
        for word_break in [
            crate::WordBreak::Normal,
            crate::WordBreak::KeepAll,
            crate::WordBreak::BreakWord,
            crate::WordBreak::Manual,
        ] {
            for (text, expected) in [
                ("กก12", vec![]),
                ("12กก", vec![]),
                ("กกאב", vec![]),
                ("אבกก", vec![]),
                ("กก.กก", vec![]),
                ("(  กก", vec![]),
                ("กก) กก", vec![4]),
                ("กก กก", vec![3]),
            ] {
                assert_eq!(
                    ordinary_boundaries(text, word_break, LineBreak::Auto)
                        .iter()
                        .enumerate()
                        .filter_map(|(atom, boundary)| (*boundary == Boundary::Line).then_some(atom))
                        .collect::<Vec<_>>(),
                    expected,
                    "{text:?}"
                );
            }
        }
    }

    #[test]
    fn ordinary_sa_mark_inherits_glue_before_extra_loose_iteration_cut() {
        let classes = CodePointMapData::<Class>::new();
        let categories = CodePointMapData::<GeneralCategory>::new();
        let graphemes = CodePointMapData::<icu_properties::props::GraphemeClusterBreak>::new();
        let glue_mark = classes
            .iter_ranges_for_value(Class::Glue)
            .flatten()
            .find(|cp| {
                categories.get32(*cp) == GeneralCategory::NonspacingMark
                    && graphemes.get32(*cp) == icu_properties::props::GraphemeClusterBreak::Extend
            })
            .and_then(char::from_u32)
            .unwrap();
        let text = format!("a{glue_mark}\u{e48}々");
        // SA Mn resolves to CM, so LB9 retains GL despite ICU's early Loose
        // return for the named iteration mark, including BreakAll's ID mode.
        for word_break in [
            crate::WordBreak::Normal,
            crate::WordBreak::KeepAll,
            crate::WordBreak::BreakWord,
            crate::WordBreak::Manual,
            crate::WordBreak::BreakAll,
        ] {
            assert_ne!(
                ordinary_boundaries(&text, word_break, LineBreak::Loose)[3],
                Boundary::Line
            );
        }
    }

    #[test]
    fn provider_admission_keeps_lb8_before_glue_but_not_before_hard_or_zw() {
        for (text, seam, allowed) in [
            ("กก\u{200b} \u{a0}ก", 4, true),
            ("กก\u{200b} \u{2060}ก", 4, true),
            ("กก\u{200b} \nก", 4, false),
            ("กก\u{200b} \u{200b}ก", 4, false),
            // LB9 excludes prior SP; LB8a does not prohibit before ZWJ.
            ("กก \u{200d}ก", 3, true),
        ] {
            let source = crate::Source::new(text);
            let projection = SourceProjection::new(&source).unwrap();
            let styles = vec![AnalyzedStyleRun {
                owner: crate::engine::resolve::AnalyzedStyleOwner::Source(crate::Range::new(
                    0,
                    source.text().len(),
                )),
                style: crate::ValidatedStyle::try_from(crate::Style::default()).unwrap(),
            }];
            let offsets: Vec<_> = projection
                .segmentation
                .char_indices()
                .map(|(byte, _)| byte)
                .chain(std::iter::once(projection.segmentation.len()))
                .collect();
            let mut boundaries = vec![Boundary::None; projection.atoms.len() + 1];
            apply(
                &projection,
                &styles,
                &policies(&styles).unwrap(),
                &offsets,
                0..projection.atoms.len(),
                &mut boundaries,
            );
            // The actual ZW SP* prefix permits WJ/GL, but LB6/LB7 win first.
            // Inspect this phase directly, before prepared's hard-break guard.
            assert_eq!(boundaries[seam] == Boundary::Line, allowed, "{text:?}");
        }
    }

    #[test]
    fn terminal_preserved_separator_retains_context_and_incident_prohibitions() {
        // CSS's maximal preserved separator cause retains UAX55 LB11/13/14/
        // 15a. The SPACE-before-GL exception remains distinct from WJ/ZWJ.
        for (text, seam, allowed) in [
            ("a b", 2, true),
            ("(  A", 3, false),
            ("“  A", 3, false),
            ("a \u{2060}b", 2, false),
            ("a \u{200d}b", 2, false),
            ("a \u{a0}b", 2, true),
            ("a )", 2, false),
            ("a  b", 2, false),
        ] {
            let source = crate::Source::new(text);
            let projection = SourceProjection::new(&source).unwrap();
            let styles = vec![AnalyzedStyleRun {
                owner: crate::engine::resolve::AnalyzedStyleOwner::Source(crate::Range::new(
                    0,
                    source.text().len(),
                )),
                style: crate::ValidatedStyle::try_from(crate::Style::default()).unwrap(),
            }];
            let offsets: Vec<_> = projection
                .segmentation
                .char_indices()
                .map(|(byte, _)| byte)
                .chain(std::iter::once(projection.segmentation.len()))
                .collect();
            let mut boundaries = vec![Boundary::None; projection.atoms.len() + 1];
            apply(
                &projection,
                &styles,
                &policies(&styles).unwrap(),
                &offsets,
                0..seam,
                &mut boundaries,
            );
            assert_eq!(boundaries[seam] == Boundary::Line, allowed, "{text:?}");
        }
    }
}
