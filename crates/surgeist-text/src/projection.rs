use std::collections::BTreeMap;

use icu_segmenter::GraphemeClusterSegmenter;

use super::{Affinity, Brush, Error, ErrorCode, ErrorDetail, Range, Result, Source, Text};

/// A byte index in the private scalar provider projection.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ProviderIndex(pub(crate) usize);

#[derive(Clone, Debug)]
pub(crate) struct ScalarProjection {
    pub(crate) text: String,
    // Every scalar boundary, including the final boundary. Pair interiors are absent.
    offsets: Vec<(usize, ProviderIndex)>,
    pub(crate) graphemes: Vec<usize>,
    stops: Vec<usize>,
    lines: Vec<Range>,
    // Parley shapes a private space solely to establish empty-source cursor metrics.
    empty_metrics: bool,
}

impl ScalarProjection {
    pub(crate) fn admit(source: &Source) -> Result<()> {
        let mut index = 0;
        for scalar in char::decode_utf16(source.text().code_units().iter().copied()) {
            match scalar {
                Ok(ch) => index += ch.len_utf16(),
                Err(error) => {
                    return Err(Error::new(
                        ErrorCode::UnsupportedFeature,
                        "isolated-surrogate layout requires original-source analysis",
                    )
                    .with_detail(ErrorDetail::IsolatedSurrogateLayout {
                        index,
                        unit: error.unpaired_surrogate(),
                    }));
                }
            }
        }
        for span in source.spans() {
            Self::admit_boundary(
                source.text(),
                span.range().start,
                super::ShapingBoundary::SpanStart,
            )?;
            Self::admit_boundary(
                source.text(),
                span.range().end,
                super::ShapingBoundary::SpanEnd,
            )?;
        }
        for box_ in source.boxes() {
            Self::admit_boundary(
                source.text(),
                box_.index(),
                super::ShapingBoundary::InlineBox,
            )?;
        }
        Ok(())
    }

    fn admit_boundary(text: &Text, index: usize, kind: super::ShapingBoundary) -> Result<()> {
        if !text.is_scalar_boundary(index) {
            return Err(Error::new(
                ErrorCode::UnsupportedFeature,
                "shaping boundary is inside a UTF-16 surrogate pair",
            )
            .with_detail(ErrorDetail::UnsupportedShapingBoundary { index, kind }));
        }
        Ok(())
    }

    pub(crate) fn new(source: &Source) -> Result<Self> {
        Self::admit(source)?;
        let mut text = String::new();
        let mut offsets = Vec::new();
        let mut unit = 0;
        for scalar in char::decode_utf16(source.text().code_units().iter().copied()) {
            let scalar = scalar.expect("scalar admission established well-formed source");
            offsets.push((unit, ProviderIndex(text.len())));
            text.len()
                .checked_add(scalar.len_utf8())
                .ok_or_else(length_overflow)?;
            text.push(scalar);
            unit += scalar.len_utf16();
        }
        offsets.push((unit, ProviderIndex(text.len())));
        let mut projection = Self {
            text,
            offsets,
            graphemes: Vec::new(),
            stops: Vec::new(),
            lines: Vec::new(),
            empty_metrics: source.text().is_empty() && source.boxes().is_empty(),
        };
        projection.graphemes = GraphemeClusterSegmenter::new()
            .segment_str(&projection.text)
            .map(|byte| projection.unit(ProviderIndex(byte)))
            .collect::<Result<Vec<_>>>()?;
        Ok(projection)
    }

    pub(crate) fn byte(&self, unit: usize) -> ProviderIndex {
        let index = self
            .offsets
            .binary_search_by_key(&unit, |(unit, _)| *unit)
            .expect("layout admission or caret projection established scalar boundary");
        self.offsets[index].1
    }

    pub(crate) fn unit(&self, byte: ProviderIndex) -> Result<usize> {
        if self.empty_metrics && byte.0 == 1 {
            return Ok(0);
        }
        self.offsets
            .binary_search_by_key(&byte, |(_, byte)| *byte)
            .map(|index| self.offsets[index].0)
            .map_err(|_| {
                Error::new(
                    ErrorCode::LayoutFailed,
                    "provider returned an invalid source boundary",
                )
                .with_detail(ErrorDetail::ProviderSourceRange {
                    start: byte.0,
                    end: byte.0,
                })
            })
    }

    pub(crate) fn range(&self, bytes: std::ops::Range<usize>) -> Range {
        Range::new(
            self.unit(ProviderIndex(bytes.start))
                .expect("provider ranges were validated"),
            self.unit(ProviderIndex(bytes.end))
                .expect("provider ranges were validated"),
        )
    }

    pub(crate) fn finish(&mut self, layout: &parley::Layout<Brush>) -> Result<()> {
        let mut indivisible = Vec::new();
        let anchors: BTreeMap<_, _> = layout
            .inline_boxes()
            .iter()
            .map(|box_| (box_.id, box_.index))
            .collect();
        for line in layout.lines() {
            self.lines.push(self.source_line_range(&line, &anchors)?);
            for run in line.runs() {
                self.validate_range(run.text_range())?;
                for cluster in run.clusters() {
                    let range = cluster.text_range();
                    self.validate_range(range.clone())?;
                    indivisible.push(self.range(range));
                }
            }
        }
        self.stops = legal_stops(&self.graphemes, indivisible);
        Ok(())
    }

    pub(crate) fn line_range(&self, index: usize) -> Range {
        self.lines[index]
    }

    fn source_line_range(
        &self,
        line: &parley::Line<'_, Brush>,
        anchors: &BTreeMap<u64, usize>,
    ) -> Result<Range> {
        let range = line.text_range();
        // Parley initializes each line start to MAX and only text runs update it.
        // An actual all-box line therefore has no text extent. Its source position
        // is established by the boxes' original anchors, never by clamping MAX.
        if range.start == usize::MAX
            && range.end == 0
            && !line.is_empty()
            && line.runs().next().is_none()
        {
            let mut anchor = None;
            for item in line.items() {
                let parley::PositionedLayoutItem::InlineBox(box_) = item else {
                    return Err(provider_range_error(range));
                };
                let Some(&index) = anchors.get(&box_.id) else {
                    return Err(provider_range_error(range));
                };
                if anchor.is_some_and(|previous| previous != index) {
                    return Err(provider_range_error(range));
                }
                anchor = Some(index);
            }
            if let Some(anchor) = anchor {
                let unit = self.unit(ProviderIndex(anchor))?;
                return Ok(Range::new(unit, unit));
            }
        }
        self.validate_range(range.clone())?;
        Ok(self.range(range))
    }

    fn validate_range(&self, range: std::ops::Range<usize>) -> Result<()> {
        if range.start > range.end
            || self.unit(ProviderIndex(range.start)).is_err()
            || self.unit(ProviderIndex(range.end)).is_err()
        {
            return Err(provider_range_error(range));
        }
        Ok(())
    }

    pub(crate) fn caret(&self, index: usize, affinity: Affinity) -> usize {
        match self.stops.binary_search(&index) {
            Ok(_) => index,
            Err(next) => match affinity {
                Affinity::Before => self.stops[next.saturating_sub(1)],
                Affinity::After => self.stops[next.min(self.stops.len() - 1)],
            },
        }
    }

    pub(crate) fn is_stop(&self, index: usize) -> bool {
        self.stops.binary_search(&index).is_ok()
    }

    #[cfg(feature = "text-accessibility")]
    pub(crate) fn selectable_ranges(&self) -> impl Iterator<Item = Range> + '_ {
        self.stops
            .windows(2)
            .map(|pair| Range::new(pair[0], pair[1]))
    }

    pub(crate) fn selection_range(&self, range: Range) -> Range {
        Range::new(
            self.caret(range.start, Affinity::Before),
            self.caret(range.end, Affinity::After),
        )
    }
}

fn provider_range_error(range: std::ops::Range<usize>) -> Error {
    Error::new(
        ErrorCode::LayoutFailed,
        "provider returned an invalid source range",
    )
    .with_detail(ErrorDetail::ProviderSourceRange {
        start: range.start,
        end: range.end,
    })
}

fn legal_stops(graphemes: &[usize], mut intervals: Vec<Range>) -> Vec<usize> {
    intervals.sort_unstable_by_key(|range| (range.start, range.end));
    let mut intervals = intervals.iter().peekable();
    let mut covered_end = 0;
    let mut stops = Vec::new();
    for &unit in graphemes {
        while intervals.peek().is_some_and(|range| range.start < unit) {
            covered_end = covered_end.max(intervals.next().expect("peeked interval").end);
        }
        if covered_end <= unit {
            stops.push(unit);
        }
    }
    stops
}

pub(crate) fn length_overflow() -> Error {
    Error::new(
        ErrorCode::SourceLengthOverflow,
        "source length is not representable",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_stop_sweep_preserves_strict_boundaries_under_overlap_and_visual_order() {
        let graphemes: Vec<_> = (0..=8).collect();
        let intervals = vec![Range::new(5, 7), Range::new(1, 4), Range::new(2, 6)];
        assert_eq!(legal_stops(&graphemes, intervals), [0, 1, 7, 8]);
        assert_eq!(legal_stops(&graphemes, Vec::new()), graphemes);
    }
}
